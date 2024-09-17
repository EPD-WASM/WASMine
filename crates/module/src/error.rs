#[derive(Debug, thiserror::Error)]
pub enum ModuleError {
    #[error("Module error: {0}")]
    Msg(String),
    #[error("Parser not set")]
    ParserNotSet,
    #[error("Missing loader datasource")]
    MissingLoaderDatasource,
    #[error("ResourceBuffer error: {0}")]
    ResourceBufferError(#[from] resource_buffer::ResourceBufferError),
    #[error("Serializer error: {0}")]
    SerializerError(#[from] rkyv::ser::serializers::BufferSerializerError),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
