mod discovery;
mod matcher;
mod registry;

pub use discovery::{RouteEntry, discover_routes, file_to_route_path};
pub use matcher::{route_to_axum_path, matches as match_route};
pub use registry::{RouteRegistry, SharedRegistry, new_shared_registry};
