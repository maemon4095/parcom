use parcom_streams_core::StreamSource;

pub struct GenericStream<S: StreamSource> {
    source: S,
}
