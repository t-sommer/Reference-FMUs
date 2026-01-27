use std::{fs::File, io::Write};
use zip::ZipArchive;
use tempfile::TempDir;

use crate::{fmi3::FMU3, model_description::{Dimension, ModelVariable, VariableType}};


pub fn extract_fmu(fmu_path: &str) -> Result<TempDir, Box<dyn std::error::Error>> {

    // Create temporary directory
    let temp_dir = TempDir::new()?;

    // Open the FMU file (which is a ZIP archive)
    let file = File::open(fmu_path)?;
    let mut archive = ZipArchive::new(file)?;

    // Extract all files
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => temp_dir.path().join(path),
            None => continue,
        };

        if (*file.name()).ends_with('/') {
            // Directory
            std::fs::create_dir_all(&outpath)?;
        } else {
            // File
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(p)?;
                }
            }
            let mut outfile = File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    Ok(temp_dir)
}

pub fn write_header(variables: &[&ModelVariable], stream: &mut dyn Write) -> std::io::Result<()> {
    write!(stream, "\"time\"")?;
    for variable in variables {
        write!(stream, ",\"{}\"", variable.name)?;
    }
    writeln!(stream)?;
    Ok(())
}

macro_rules! write_values {
    ($values:expr, $stream:expr) => {{
        for (i, value) in $values.iter().enumerate() {
            if i > 0 {
                write!($stream, " ")?;
            }
            write!($stream, "{value}")?;
        }}  
    };
}

pub struct Recorder<'a> {
    pub variables: Vec<&'a ModelVariable>,
    pub stream: &'a mut dyn Write,
    pub fmu: &'a FMU3<'a>,
    pub sizes: Vec<usize>,
}

impl Recorder<'_> {

    pub fn write_header(&mut self) -> std::io::Result<()> {
        write!(self.stream, "\"time\"")?;
        for variable in self.variables.iter() {
            write!(self.stream, ",\"{}\"", variable.name)?;
        }
        writeln!(self.stream)?;
        Ok(())
    }

    pub fn update_sizes(&mut self) {
        self.sizes.clear();
        for variable in self.variables.iter() {
            let mut size = 1usize;
            for dimension in variable.dimensions.iter() {
                size *= match dimension {
                    Dimension::Fixed(value) => *value,
                    Dimension::Variable(value_reference) => {
                        let mut values = [0u64];
                        self.fmu.getUInt64(&[*value_reference], &mut values);
                        values[0] as usize
                    },
                };
            }
            self.sizes.push(size);
        }
    }

    pub fn sample(&mut self, time: f64) -> std::io::Result<()> {

        if self.sizes.is_empty() {
            self.update_sizes();
        }

        write!(self.stream, "{time}")?;
        
        for (i, variable) in self.variables.iter().enumerate() {
            let size = self.sizes.get(i).unwrap();
            write!(self.stream, ",")?;
            let value_references = [variable.valueReference];
            match variable.variableType {
                VariableType::Float32 => {
                    let mut values = vec![0f32; *size];
                    self.fmu.getFloat32(&value_references, &mut values);
                    write_values!(values, self.stream)
                },
                VariableType::Float64 => {
                    let mut values = vec![0f64; *size];
                    self.fmu.getFloat64(&value_references, &mut values);
                    write_values!(values, self.stream)
                },
                _ => todo!(),
            }
        }
        writeln!(self.stream)?;
        Ok(())
    }

}

pub fn sample(time: f64, variables: &[&ModelVariable], fmu: &FMU3, stream: &mut dyn Write) -> std::io::Result<()> {
    write!(stream, "{time}")?;
    for variable in variables {
        write!(stream, ",")?;
        let value_references = [variable.valueReference];
        match variable.variableType {
            VariableType::Float32 => {
                let mut values = [0.0f32];
                fmu.getFloat32(&value_references, &mut values);
                write_values!(values, stream)
            },
            VariableType::Float64 => {
                let mut values = [0.0];
                fmu.getFloat64(&value_references, &mut values);
                write_values!(values, stream)
            },
            _ => todo!(),
        }
    }
    writeln!(stream)?;
    Ok(())
}