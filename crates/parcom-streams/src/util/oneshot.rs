#[cfg(test)]
use super::trace_cell::TraceCell;
use std::{
    mem::ManuallyDrop,
    sync::atomic::{AtomicUsize, Ordering},
    thread::Thread,
};

pub fn oneshot_channel<T>() -> (Sender<T>, Receiver<T>) {
    let slot = Slot {
        value: SlotValue { uninit: () },
        state: AtomicUsize::new(STATE_INITIAL),
    };

    let slot = Box::into_raw(Box::new(
        #[cfg(test)]
        {
            TraceCell::new(slot)
        },
        #[cfg(not(test))]
        {
            slot
        },
    ));

    let sender = Sender { slot };
    let receiver = Receiver { slot };

    (sender, receiver)
}

#[derive(Debug)]
pub struct Sender<T> {
    #[cfg(test)]
    slot: *mut TraceCell<Slot<T>>,
    #[cfg(not(test))]
    slot: *mut Slot<T>,
}

#[derive(Debug)]
pub struct Receiver<T> {
    #[cfg(test)]
    slot: *mut TraceCell<Slot<T>>,
    #[cfg(not(test))]
    slot: *mut Slot<T>,
}

union SlotValue<T> {
    uninit: (),
    message: ManuallyDrop<T>,
    thread: ManuallyDrop<Thread>,
}

/// Sender: alive + before send, Receiver: alive + before receive
const STATE_INITIAL: usize = 0;

const STATE_MODIFYING: usize = 1;

/// Sender: alive + before send, Receiver: alive + receiving
const STATE_WAITING: usize = 2;

/// Sender: dead + after send, Receiver: alive + before receive
const STATE_READY: usize = 3;

/// Sender: dead + before send, Receiver: alive + before receive
const STATE_SENDER_DROPPED_BEFORE_SEND: usize = 4;

/// Sender: alive + before send, Receiver: dead + before receive
const STATE_RECEIVER_DROPPED_BEFORE_RECV: usize = 5;

/// Sender: dead, Receiver: dead
const STATE_DEAD: usize = usize::MAX;

struct Slot<T> {
    /// - `state`が`STATE_INITIAL`の場合、`value`には値がセットされていない。
    /// - `state`が`STATE_MODIFYING`の場合、`value`の値は未定義
    /// - `state`が`STATE_WAITING`の場合、`value`には`Thread`型の値がセットされている。
    /// - `state`が`STATE_READY`の場合、`value`には`T`型の値がセットされている。
    /// - `state`が`STATE_SENDER_DROPPED_BEFORE_SEND`の場合、`value`には値がセットされていない。
    /// - `state`が`STATE_RECEIVER_DROPPED_BEFORE_RECV`の場合、`value`には値がセットされていない。
    /// - `state`が`STATE_DEAD`の場合、`value`には値がセットされていない。
    value: SlotValue<T>,
    state: AtomicUsize,
}

impl<T> Slot<T> {
    /// `Slot`のロックを獲得し、`Slot`のステートを返す。
    fn acquire_lock(&self) -> usize {
        let mut state = self.state.load(Ordering::Relaxed);
        loop {
            if state == STATE_MODIFYING {
                state = self.state.load(Ordering::Relaxed);
            } else {
                let result = self.state.compare_exchange_weak(
                    state,
                    STATE_MODIFYING,
                    Ordering::Acquire,
                    Ordering::Relaxed,
                );

                match result {
                    Ok(v) => break v,
                    Err(e) => state = e,
                }
            }

            std::hint::spin_loop();
        }
    }

    fn release_lock(&self, state: usize) {
        self.state.store(state, Ordering::Release);
    }
}

unsafe impl<T: Send> Send for Sender<T> {}

impl<T> Sender<T> {
    pub fn send(self, value: T) -> Result<(), T> {
        let state = unsafe { (&*self.slot).acquire_lock() };
        match state {
            STATE_INITIAL => unsafe {
                (&mut *self.slot).value.message = ManuallyDrop::new(value);
                (&*self.slot).release_lock(STATE_READY);
                std::mem::forget(self);
                Ok(())
            },
            STATE_WAITING => unsafe {
                let old_value = std::mem::replace(
                    &mut (&mut *self.slot).value,
                    SlotValue {
                        message: ManuallyDrop::new(value),
                    },
                );
                let thread = ManuallyDrop::into_inner(old_value.thread);
                (&*self.slot).release_lock(STATE_READY);
                thread.unpark();
                std::mem::forget(self);
                Ok(())
            },
            STATE_RECEIVER_DROPPED_BEFORE_RECV => unsafe {
                (&*self.slot).release_lock(STATE_DEAD);
                drop(Box::from_raw(self.slot));
                std::mem::forget(self);
                Err(value)
            },
            v => unreachable!("send invalid state: {}", v),
        }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        let state = unsafe { (&*self.slot).acquire_lock() };
        match state {
            STATE_INITIAL => unsafe {
                (&*self.slot).release_lock(STATE_SENDER_DROPPED_BEFORE_SEND);
            },
            STATE_WAITING => unsafe {
                let value =
                    std::mem::replace(&mut (&mut *self.slot).value, SlotValue { uninit: () });
                let thread = ManuallyDrop::into_inner(value.thread);
                (&*self.slot).release_lock(STATE_SENDER_DROPPED_BEFORE_SEND);
                thread.unpark();
            },
            STATE_RECEIVER_DROPPED_BEFORE_RECV => unsafe {
                (&*self.slot).release_lock(STATE_DEAD);
                drop(Box::from_raw(self.slot));
            },
            v => unreachable!("sender drop invalid state: {}", v),
        }
    }
}

unsafe impl<T: Send> Send for Receiver<T> {}

impl<T> Receiver<T> {
    pub fn recv(self) -> Result<T, RecvError> {
        loop {
            let state = unsafe { (&*self.slot).acquire_lock() };
            match state {
                STATE_INITIAL => unsafe {
                    (&mut *self.slot).value.thread = ManuallyDrop::new(std::thread::current());
                    (&*self.slot).release_lock(STATE_WAITING);
                    std::thread::park();
                },
                STATE_READY => unsafe {
                    let value =
                        std::mem::replace(&mut (&mut *self.slot).value, SlotValue { uninit: () });

                    (&*self.slot).release_lock(STATE_DEAD);
                    drop(Box::from_raw(self.slot));
                    std::mem::forget(self);

                    break Ok(ManuallyDrop::into_inner(value.message));
                },
                STATE_SENDER_DROPPED_BEFORE_SEND => unsafe {
                    (&*self.slot).release_lock(STATE_DEAD);
                    drop(Box::from_raw(self.slot));
                    std::mem::forget(self);
                    break Err(RecvError::SenderDropped);
                },
                STATE_WAITING => unsafe {
                    (&*self.slot).release_lock(STATE_WAITING);
                    std::thread::park();
                },
                v => unreachable!("recv invalid state: {}", v),
            }
        }
    }

    pub fn try_recv(self) -> Result<T, TryRecvError<T>> {
        let state = unsafe { (&*self.slot).acquire_lock() };
        match state {
            STATE_INITIAL => unsafe {
                (&*self.slot).release_lock(STATE_INITIAL);
                Err(TryRecvError::NotReady(self))
            },
            STATE_READY => unsafe {
                let value =
                    std::mem::replace(&mut (&mut *self.slot).value, SlotValue { uninit: () });

                (&*self.slot).release_lock(STATE_DEAD);
                drop(Box::from_raw(self.slot));
                std::mem::forget(self);

                Ok(ManuallyDrop::into_inner(value.message))
            },
            STATE_SENDER_DROPPED_BEFORE_SEND => unsafe {
                (&*self.slot).release_lock(STATE_DEAD);
                drop(Box::from_raw(self.slot));
                std::mem::forget(self);

                Err(TryRecvError::SenderDropped)
            },
            STATE_WAITING => unsafe {
                (&*self.slot).release_lock(STATE_WAITING);
                Err(TryRecvError::NotReady(self))
            },
            v => unreachable!("try recv invalid state: {}", v),
        }
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        let state = unsafe { (&*self.slot).acquire_lock() };
        match state {
            STATE_READY => unsafe {
                ManuallyDrop::drop(&mut (&mut *self.slot).value.message);
                (&*self.slot).release_lock(STATE_DEAD);
                drop(Box::from_raw(self.slot));
            },
            STATE_INITIAL => unsafe {
                (&*self.slot).release_lock(STATE_RECEIVER_DROPPED_BEFORE_RECV);
            },
            STATE_SENDER_DROPPED_BEFORE_SEND => unsafe {
                (&*self.slot).release_lock(STATE_DEAD);
                drop(Box::from_raw(self.slot));
            },
            v => unreachable!("sender drop invalid state: {}", v),
        }
    }
}

#[derive(Debug)]
pub enum RecvError {
    SenderDropped,
}

#[derive(Debug)]
pub enum TryRecvError<T> {
    NotReady(Receiver<T>),
    SenderDropped,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::util::trace_cell::{TraceStoreTracing, TRACE_STORE};
    use std::sync::{Arc, Barrier};

    fn assert_send<T: Send>() {}

    #[allow(unused)]
    fn assert_sender_is_send_for_any_send_type<T: Send>() {
        assert_send::<Sender<T>>()
    }

    #[allow(unused)]
    fn assert_receiver_is_send_for_any_send_type<T: Send>() {
        assert_send::<Receiver<T>>()
    }

    #[test]
    fn test_recv_success() {
        let tracing = TRACE_STORE.start_tracing();
        for _ in 0..0xFFFF {
            test_recv_success_once(&tracing);
        }

        let allocs = tracing.allocs();
        let drops = tracing.drops();
        assert_eq!(allocs, drops)
    }

    fn test_recv_success_once(tracing: &TraceStoreTracing) {
        let test_value: u32 = rand::random();

        let (sender, receiver) = oneshot_channel::<TraceCell<u32>>();
        let barrier = Arc::new(Barrier::new(2));

        let receiver_thread = std::thread::spawn({
            let barrier = barrier.clone();
            move || {
                barrier.wait();
                receiver.recv()
            }
        });

        let sender_thread = std::thread::spawn(move || {
            barrier.wait();
            sender.send(TraceCell::new(test_value))
        });

        let receiver_result = receiver_thread.join().unwrap();
        let sender_result = sender_thread.join().unwrap();

        match (receiver_result, sender_result) {
            (Ok(v), Ok(_)) => {
                assert_eq!(*v, test_value);
            }
            _ => unreachable!(),
        }

        let allocs = tracing.allocs();
        let drops = tracing.drops();
        assert_eq!(allocs, drops)
    }

    #[test]
    fn test_recv_fail() {
        let tracing = TRACE_STORE.start_tracing();
        for _ in 0..0xFFFF {
            test_recv_fail_once(&tracing);
        }

        let allocs = tracing.allocs();
        let drops = tracing.drops();
        assert_eq!(allocs, drops)
    }

    fn test_recv_fail_once(tracing: &TraceStoreTracing) {
        let (sender, receiver) = oneshot_channel::<TraceCell<u32>>();
        let barrier = Arc::new(Barrier::new(2));

        let receiver_thread = std::thread::spawn({
            let barrier = barrier.clone();
            move || {
                barrier.wait();
                let value = receiver.recv().unwrap_err();
                assert!(matches!(value, RecvError::SenderDropped));
            }
        });

        let sender_thread = std::thread::spawn(move || {
            barrier.wait();
            drop(sender);
        });

        receiver_thread.join().unwrap();
        sender_thread.join().unwrap();

        let allocs = tracing.allocs();
        let drops = tracing.drops();
        assert_eq!(allocs, drops)
    }

    #[test]
    fn test_drop_recv() {
        let tracing = TRACE_STORE.start_tracing();

        for _ in 0..0xFFFF {
            test_drop_recv_once(&tracing);
        }

        let allocs = tracing.allocs();
        let drops = tracing.drops();
        assert_eq!(allocs, drops)
    }

    fn test_drop_recv_once(tracing: &TraceStoreTracing) {
        let test_value: u32 = rand::random();

        let (sender, receiver) = oneshot_channel::<TraceCell<u32>>();
        let barrier = Arc::new(Barrier::new(2));

        let receiver_thread = std::thread::spawn({
            let barrier = barrier.clone();
            move || {
                barrier.wait();
                drop(receiver);
            }
        });

        let sender_thread = std::thread::spawn(move || {
            barrier.wait();
            let v = sender.send(TraceCell::new(test_value));

            if let Err(v) = v {
                assert_eq!(*v, test_value);
            }
        });

        receiver_thread.join().unwrap();
        sender_thread.join().unwrap();

        let allocs = tracing.allocs();
        let drops = tracing.drops();
        assert_eq!(allocs, drops)
    }

    #[test]
    fn test_drop_both() {
        let tracing = TRACE_STORE.start_tracing();
        for _ in 0..0xFFFF {
            test_drop_both_once(&tracing);
        }

        let allocs = tracing.allocs();
        let drops = tracing.drops();
        assert_eq!(allocs, drops)
    }

    fn test_drop_both_once(tracing: &TraceStoreTracing) {
        let (sender, receiver) = oneshot_channel::<TraceCell<u32>>();
        let barrier = Arc::new(Barrier::new(2));

        let receiver_thread = std::thread::spawn({
            let barrier = barrier.clone();
            move || {
                barrier.wait();
                drop(receiver);
            }
        });

        let sender_thread = std::thread::spawn(move || {
            barrier.wait();
            drop(sender);
        });

        receiver_thread.join().unwrap();
        sender_thread.join().unwrap();

        let allocs = tracing.allocs();
        let drops = tracing.drops();

        assert_eq!(allocs, drops)
    }

    #[test]
    fn test_try_recv_return_not_ready_before_send_and_return_ok_after_send() {
        let (sender, mut receiver) = oneshot_channel::<TraceCell<u32>>();

        receiver = match receiver.try_recv() {
            Err(TryRecvError::NotReady(v)) => v,
            _ => unreachable!(),
        };

        let test_value: u32 = rand::random();
        sender.send(TraceCell::new(test_value)).unwrap();

        match receiver.try_recv() {
            Ok(v) => assert_eq!(*v, test_value),
            _ => unreachable!(),
        }
    }
}
