use crate::Cluster;

/// A handle to a parsed WebAssembly module stored in a cluster
pub struct ModuleHandle<'cluster> {
    cluster: &'cluster Cluster,
}
