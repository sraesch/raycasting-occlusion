use log::error;
use nalgebra_glm::Mat4;
use serde::{Deserialize, Serialize};

use crate::{Error, Result};

/// The configuration for the test
#[derive(Debug, Deserialize, Serialize)]
pub struct TestConfig {
    /// The occlusion setups, i.e., the different configurations to test the occlusion.
    pub setups: Vec<OcclusionSetup>,

    /// The input files for the testing.
    /// Can be expressions like `*.glb`
    pub input: Vec<String>,

    /// The views to use for the tests
    pub views: Vec<View>,
}

impl TestConfig {
    /// Reads the configuration from the provided reader.
    ///
    /// # Arguments
    /// * `reader` - The reader to read the configuration from.
    pub fn read<R: std::io::Read>(mut reader: R) -> Result<Self> {
        // read everything from the reader into a string
        let mut toml = String::new();
        reader.read_to_string(&mut toml)?;

        // deserialize into the test config
        let config: TestConfig = toml::from_str(&toml).map_err(|e| {
            error!("Failed to parse the configuration: {:?}", e);

            Error::DeserializationError(Box::new(e))
        })?;

        Ok(config)
    }

    /// Writes the configuration to the provided writer.
    ///
    /// # Arguments
    /// * `writer` - The writer to write the configuration to.
    pub fn write<W: std::io::Write>(&self, mut writer: W) -> Result<()> {
        // serialize the configuration into a string
        let toml = toml::to_string_pretty(&self).map_err(|e| {
            error!("Failed to serialize the configuration: {:?}", e);

            Error::SerializationError(Box::new(e))
        })?;

        // write the string to the writer
        writer.write_all(toml.as_bytes())?;

        Ok(())
    }
}

/// A camera view defined by its view and projection matrix.
#[derive(Debug, Deserialize, Serialize)]
pub struct View {
    /// The view matrix
    pub view_matrix: Mat4,

    /// The projection matrix
    pub projection_matrix: Mat4,
}

/// The occlusion tester
#[derive(Debug, Deserialize, Serialize)]
pub enum OcclusionSetup {
    Rasterizer(RasterizerOptions),
}

/// The options for a rasterizer occlusion test
#[derive(Debug, Deserialize, Serialize)]
pub struct RasterizerOptions {
    /// The output file
    pub frame_size: usize,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_loading_config() {
        let simple_config_data = include_bytes!("../../examples/configs/simple.toml");
        let config = TestConfig::read(&simple_config_data[..]).unwrap();
    }
}
