use pin_project::pin_project;

#[pin_project(project = PinOptionProj)]
pub enum PinOption<T> {
    Some(#[pin] T),
    None,
}
