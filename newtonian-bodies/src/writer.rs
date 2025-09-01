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


#[cfg(test)]
mod tests {  
    use super::*;
    use crate::body::Vector;
    use parquet::arrow::arrow_reader::ParquetRecordBatchReader;
    use arrow::record_batch::RecordBatchReader;
    use arrow::array::{Float64Array, StringArray, UInt64Array};

    fn create_test_body(name: &str, mass: f64, x: f64, y: f64, z: f64) -> Body {
        Body {
            name: name.to_string(),
            mass,
            position: Vector { x, y, z },
            velocity: Vector::null(),
            acceleration: Vector::null(),
        }
    }

    #[test]
    fn test_generated_file_has_the_correct_schema() {
        let test_file = PathBuf::from("test_schema.parquet");
        
        // Create writer and write test data
        let mut writer = Writer::new(test_file.clone()).unwrap();
        writer.add(0, &[create_test_body("Earth", 5.972e24, 1.496e11, 0.0, 0.0)]).unwrap();
        writer.close().unwrap();

        // Read the file and verify schema
        let file = File::open(&test_file).unwrap();
        let reader = ParquetRecordBatchReader::try_new(file, 1024).unwrap();
        let schema = reader.schema();
        
        // Check field count
        assert_eq!(schema.fields().len(), 6);
        
        // Check field names and data types
        assert_eq!(schema.field(0).name(), "time");
        assert_eq!(schema.field(0).data_type(), &DataType::UInt64);
        assert!(!schema.field(0).is_nullable());
        
        assert_eq!(schema.field(1).name(), "name");
        assert_eq!(schema.field(1).data_type(), &DataType::Utf8);
        assert!(!schema.field(1).is_nullable());
        
        assert_eq!(schema.field(2).name(), "mass");
        assert_eq!(schema.field(2).data_type(), &DataType::Float64);
        assert!(!schema.field(2).is_nullable());
        
        assert_eq!(schema.field(3).name(), "pos_x");
        assert_eq!(schema.field(3).data_type(), &DataType::Float64);
        assert!(!schema.field(3).is_nullable());
        
        assert_eq!(schema.field(4).name(), "pos_y");
        assert_eq!(schema.field(4).data_type(), &DataType::Float64);
        assert!(!schema.field(4).is_nullable());
        
        assert_eq!(schema.field(5).name(), "pos_z");
        assert_eq!(schema.field(5).data_type(), &DataType::Float64);
        assert!(!schema.field(5).is_nullable());
        
        // Clean up test file
        std::fs::remove_file(&test_file).unwrap();
    }

    #[test]
    fn test_generated_file_has_the_correct_data() {
        let test_file = PathBuf::from("test_data.parquet");
        let mut writer = Writer::new(test_file.clone()).unwrap();
        writer.add(0, &[create_test_body("Earth", 5.972e24, 1.496e11, 0.0, 0.0)]).unwrap();
        writer.close().unwrap();

        let file = File::open(&test_file).unwrap();
        let mut reader = ParquetRecordBatchReader::try_new(file, 1024).unwrap();
        
        // Get the first (and only) batch
        let batch = reader.next()
            .expect("Should have at least one batch")
            .expect("Batch should be valid");
        
        // Check row count
        assert_eq!(batch.num_rows(), 1, "Should have exactly one row");
        
        // Extract arrays and verify values
        let time_array = batch.column(0).as_any()
            .downcast_ref::<UInt64Array>()
            .expect("Column 0 should be UInt64Array");
        assert_eq!(time_array.value(0), 0, "Time should be 0");
        
        let name_array = batch.column(1).as_any()
            .downcast_ref::<StringArray>()
            .expect("Column 1 should be StringArray");
        assert_eq!(name_array.value(0), "Earth", "Name should be 'Earth'");
        
        let mass_array = batch.column(2).as_any()
            .downcast_ref::<Float64Array>()
            .expect("Column 2 should be Float64Array");
        assert_eq!(mass_array.value(0), 5.972e24, "Mass should be 5.972e24");
        
        let pos_x_array = batch.column(3).as_any()
            .downcast_ref::<Float64Array>()
            .expect("Column 3 should be Float64Array");
        assert_eq!(pos_x_array.value(0), 1.496e11, "Position X should be 1.496e11");
        
        let pos_y_array = batch.column(4).as_any()
            .downcast_ref::<Float64Array>()
            .expect("Column 4 should be Float64Array");
        assert_eq!(pos_y_array.value(0), 0.0, "Position Y should be 0.0");
        
        let pos_z_array = batch.column(5).as_any()
            .downcast_ref::<Float64Array>()
            .expect("Column 5 should be Float64Array");
        assert_eq!(pos_z_array.value(0), 0.0, "Position Z should be 0.0");
        
        // Verify there are no more batches
        assert!(reader.next().is_none(), "Should have only one batch");
        
        // Clean up test file
        std::fs::remove_file(&test_file).unwrap();
    }

}