use crate::{
    task_spawner::TaskSpawner,
    util::{
        into_future_fn::into_future_fn,
        notify::{Signal, Wait},
        oneshot::{oneshot_channel, Receiver, Sender},
        pin_option::PinOption,
        pin_option::PinOptionProj,
    },
};
use parcom_streams_core::{StreamDriver, StreamDriverSession, StreamLoader};
use pin_project::pin_project;
use std::{
    future::Future,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    task::Poll,
};

pub struct ConcurrentDriver<S: TaskSpawner> {
    resume_loading_threshold: usize,
    suspend_loading_threshold: usize,
    spawner: S,
}

impl<S: TaskSpawner> ConcurrentDriver<S> {
    pub fn new(
        resume_loading_threshold: usize,
        suspend_loading_threshold: usize,
        spawner: S,
    ) -> Self {
        assert!(resume_loading_threshold < suspend_loading_threshold);
        Self {
            resume_loading_threshold,
            suspend_loading_threshold,
            spawner,
        }
    }
}

impl<S, L> StreamDriver<L> for ConcurrentDriver<S>
where
    L::Error: Send,
    L: 'static + StreamLoader + Send,
    S: TaskSpawner,
{
    type Session = ConcurrentLoader<S, L>;

    fn start(self, loader: L) -> Self::Session {
        let context = Arc::new(Context {
            resume_loading_threshold: self.resume_loading_threshold,
            suspend_loading_threshold: self.suspend_loading_threshold,
            append_data_signal: Signal::new(),
            resume_loading_signal: Signal::new(),
            unexamined_data_length: AtomicUsize::new(0),
            loader_state: AtomicUsize::new(LOADER_STATE_RUNNING),
        });

        let (sender, result_receiver) = oneshot_channel();

        let task_handle = self.spawner.spawn({
            let context = Arc::clone(&context);
            into_future_fn(move || loading_loop(context, sender, loader))
        });

        ConcurrentLoader {
            _task_handle: task_handle,
            result_receiver: Some(result_receiver),
            context,
        }
    }
}

async fn loading_loop<L>(context: Arc<Context>, sender: Sender<Result<(), L::Error>>, mut loader: L)
where
    L: StreamLoader,
{
    loop {
        let result = loader.load().await;
        match result {
            Ok(Some(load)) => {
                if load.committed() > 0 {
                    context.notify_data_append(load.committed()).await;
                }
            }
            Ok(None) => {
                let r = sender.send(Ok(()));
                context.append_data_signal.send();
                assert!(r.is_ok());
                context
                    .loader_state
                    .store(LOADER_STATE_TERMINATED_SUCCESSFULLY, Ordering::Relaxed);
                break;
            }
            Err(e) => {
                let r = sender.send(Err(e));
                context.append_data_signal.send();
                assert!(r.is_ok());
                context
                    .loader_state
                    .store(LOADER_STATE_TERMINATED_WITH_ERROR, Ordering::Relaxed);
                break;
            }
        }
    }
}

const LOADER_STATE_RUNNING: usize = 1;
const LOADER_STATE_TERMINATED_SUCCESSFULLY: usize = 2;
const LOADER_STATE_TERMINATED_WITH_ERROR: usize = 3;

struct Context {
    resume_loading_threshold: usize,
    suspend_loading_threshold: usize,
    append_data_signal: Signal,
    resume_loading_signal: Signal,
    unexamined_data_length: AtomicUsize,
    loader_state: AtomicUsize,
}

impl Context {
    fn notify_data_examined(&self, amount: usize) {
        let old = self
            .unexamined_data_length
            .fetch_sub(amount, Ordering::Relaxed);
        let new = old - amount;

        if new <= self.resume_loading_threshold {
            self.resume_loading_signal.send()
        }
    }

    fn notify_data_append(self: &Arc<Self>, amount: usize) -> WaitForDataRequest {
        let old = self
            .unexamined_data_length
            .fetch_add(amount, Ordering::Relaxed);

        let new = old + amount;

        let fut = if new < self.suspend_loading_threshold {
            PinOption::None
        } else {
            let fut = Wait::new(ResumeLoadingAccessor(self.clone()));
            if new <= self.resume_loading_threshold {
                PinOption::None
            } else {
                PinOption::Some(fut)
            }
        };

        WaitForDataRequest { fut }
    }
}

#[pin_project]
struct WaitForDataRequest {
    #[pin]
    fut: PinOption<Wait<ResumeLoadingAccessor>>,
}

impl Future for WaitForDataRequest {
    type Output = ();

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        match this.fut.as_mut().project() {
            PinOptionProj::None => Poll::Ready(()),
            PinOptionProj::Some(fut) => {
                std::task::ready!(fut.poll(cx));
                this.fut.set(PinOption::None);
                Poll::Ready(())
            }
        }
    }
}

struct ResumeLoadingAccessor(Arc<Context>);

impl std::ops::Deref for ResumeLoadingAccessor {
    type Target = Signal;

    fn deref(&self) -> &Self::Target {
        &self.0.resume_loading_signal
    }
}

pub struct ConcurrentLoader<S: TaskSpawner, H: StreamLoader> {
    _task_handle: S::Handle<()>,
    result_receiver: Option<Receiver<Result<(), H::Error>>>,
    context: Arc<Context>,
}

impl<S: TaskSpawner, H: StreamLoader> StreamDriverSession for ConcurrentLoader<S, H> {
    type Error = H::Error;
    type WaitForAppendData<'a>
        = WaitForAppendData<'a, S, H>
    where
        Self: 'a;

    fn notify_data_examined(&mut self, amount: usize) {
        self.context.notify_data_examined(amount);
    }

    fn wait_for_append_data(&mut self, _: usize) -> Self::WaitForAppendData<'_> {
        let context = Arc::clone(&self.context);
        WaitForAppendData {
            loader: self,
            fut: Wait::new(AppendDataAccessor(context)),
        }
    }

    fn is_terminated(&self) -> bool {
        self.context.loader_state.load(Ordering::Relaxed) != LOADER_STATE_RUNNING
    }
}

struct AppendDataAccessor(Arc<Context>);

impl std::ops::Deref for AppendDataAccessor {
    type Target = Signal;

    fn deref(&self) -> &Self::Target {
        &self.0.append_data_signal
    }
}

#[pin_project]
pub struct WaitForAppendData<'a, S: TaskSpawner, H: StreamLoader> {
    loader: &'a mut ConcurrentLoader<S, H>,
    #[pin]
    fut: Wait<AppendDataAccessor>,
}

impl<'a, S, H> Future for WaitForAppendData<'a, S, H>
where
    S: TaskSpawner,
    H: StreamLoader,
{
    type Output = Result<(), H::Error>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Self::Output> {
        std::task::ready!(self.as_mut().project().fut.poll(cx));

        let loader_state = self.loader.context.loader_state.load(Ordering::Relaxed);

        if loader_state == LOADER_STATE_RUNNING {
            Poll::Ready(Ok(()))
        } else {
            let receiver = self.loader.result_receiver.take().unwrap();
            let result = receiver.recv().unwrap();
            Poll::Ready(result)
        }
    }
}
