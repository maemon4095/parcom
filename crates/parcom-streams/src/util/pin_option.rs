use pin_project::pin_project;

#[pin_project(project = PinOptionProj)]
pub enum PinOption<T> {
    None,
    Some(#[pin] T),
}
