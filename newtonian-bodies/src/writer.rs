use super::dynamics::SequentialWriter;
use super::Body;
use std::error::Error;
use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;

use arrow::array::{Float64Array, StringArray, UInt64Array};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use parquet::arrow::arrow_writer::ArrowWriter;

pub struct Writer {
    writer: ArrowWriter<File>,
    schema: Schema,
}

impl Writer {
    pub fn new(file: PathBuf) -> Result<Self, Box<dyn Error>> {
        let schema = Schema::new(vec![
            Field::new("time", DataType::UInt64, false),
            Field::new("name", DataType::Utf8, false),
            Field::new("mass", DataType::Float64, false),
            Field::new("pos_x", DataType::Float64, false),
            Field::new("pos_y", DataType::Float64, false),
            Field::new("pos_z", DataType::Float64, false),
            // Add velocity and acceleration fields if needed
        ]);

        let file = File::create(file)?;
        let writer = ArrowWriter::try_new(file, Arc::new(schema.clone()), None)?;

        Ok(Self { writer, schema: schema.clone() })
    }

    // `close` is now handled when the writer is dropped, but an explicit
    // close is good practice to handle potential I/O errors.
    pub fn close(self) -> Result<(), Box<dyn Error>> {
        self.writer.close()?;
        Ok(())
    }
}

impl SequentialWriter for Writer {
    /// Converts the slice of bodies into Arrow arrays and writes them as a RecordBatch.
    fn add(&mut self, time: u64, bodies: &[Body]) -> Result<(), Box<dyn Error>> {
        let num_rows = bodies.len();

        let time_array = Arc::new(UInt64Array::from(vec![time as u64; num_rows]));
        let name_array = Arc::new(StringArray::from_iter_values(
            bodies.iter().map(|b| &b.name),
        ));
        let mass_array = Arc::new(Float64Array::from_iter_values(
            bodies.iter().map(|b| b.mass),
        ));
        let pos_x_array = Arc::new(Float64Array::from_iter_values(
            bodies.iter().map(|b| b.position.x),
        ));
        let pos_y_array = Arc::new(Float64Array::from_iter_values(
            bodies.iter().map(|b| b.position.y),
        ));
        let pos_z_array = Arc::new(Float64Array::from_iter_values(
            bodies.iter().map(|b| b.position.z),
        ));

        // 2. Create a RecordBatch from the arrays.
        let batch = RecordBatch::try_new(
            Arc::new(self.schema.clone()),
            vec![
                time_array,
                name_array,
                mass_array,
                pos_x_array,
                pos_y_array,
                pos_z_array,
            ],
        )?;

        // 3. Write the batch to the Parquet file.
        self.writer.write(&batch)?;

        Ok(())
    }
}
