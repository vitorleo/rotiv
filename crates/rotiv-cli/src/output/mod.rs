pub mod human;
pub mod json;

#[derive(Debug, Clone, Copy)]
pub enum OutputMode {
    Human,
    Json,
}
