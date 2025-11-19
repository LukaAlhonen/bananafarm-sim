use serde::{Deserialize, Serialize};
use serde_json::Error as SerdeError;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

// Represents the shape a soil moisture measurement has in the time-series database
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct SoilMoistureMeasurement {
    time: i64,
    data: f32,
    unit: String,
    id: String,
    sensor_id: String,
    location: String,
}

impl SoilMoistureMeasurement {
    pub fn new<S: Into<String>>(data: f32, unit: S, sensor_id: S, location: S) -> Self {
        SoilMoistureMeasurement {
            time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time has gone backwards")
                .as_nanos() as i64,
            data,
            id: Uuid::new_v4().to_string(),
            sensor_id: sensor_id.into(),
            unit: unit.into(),
            location: location.into(),
        }
    }

    // convert mqtt payload into SoilMoistureMeasurement
    pub fn from_payload(payload: &str) -> Result<Self, SerdeError> {
        let measurement: SoilMoistureMeasurement = serde_json::from_str(payload)?;
        Ok(measurement)
    }

    // convert SoilMoistureMeasurement into mqtt payload
    pub fn into_payload(&self) -> Result<String, SerdeError> {
        let payload = serde_json::to_string(&self)?;
        Ok(payload)
    }

    // convert SoilMoistureMeasurement into string that the influxdb3 api understands
    pub fn into_query_string(&self, table: &str) -> String {
        let query_string = format!(
            "{0},id={1},sensor_id={2},location={3},unit={4} data={5:.5} {6}",
            table, &self.id, &self.sensor_id, &self.location, &self.unit, &self.data, &self.time
        );

        query_string
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_measurement() {
        let measurement = SoilMoistureMeasurement::new(30.0, "cb", "sensor_01", "location_01");
        assert_eq!(measurement.data, 30.0);
        assert_eq!(measurement.unit, String::from("cb"));
        assert_eq!(measurement.sensor_id, String::from("sensor_01"));
        assert_eq!(measurement.location, String::from("location_01"));
    }

    #[test]
    fn test_create_measurement_from_payload() {
        let payload = r#"{
            "time": 1731240000000000000,
            "data": 30.0,
            "unit": "cb",
            "id": "id_01",
            "sensor_id": "sensor_01",
            "location": "location_01"
        }"#;

        let measurement = SoilMoistureMeasurement::from_payload(payload).unwrap();

        assert_eq!(measurement.data, 30.0);
        assert_eq!(measurement.unit, "cb");
        assert_eq!(measurement.id, "id_01");
        assert_eq!(measurement.sensor_id, "sensor_01");
        assert_eq!(measurement.location, "location_01");
    }

    #[test]
    fn test_create_payload_from_measurement() {
        let measurement = SoilMoistureMeasurement::new(30.0, "cb", "sensor_01", "location_01");

        let payload = measurement.into_payload().unwrap();

        assert_eq!(
            payload,
            format!(
                "{{\"time\":{0},\"data\":{1:.1},\"unit\":\"{2}\",\"id\":\"{3}\",\"sensor_id\":\"{4}\",\"location\":\"{5}\"}}",
                measurement.time,
                measurement.data,
                measurement.unit,
                measurement.id,
                measurement.sensor_id,
                measurement.location
            )
        )
    }

    #[test]
    fn test_create_query_string_from_measurement() {
        let measurement = SoilMoistureMeasurement::new(30.0, "cb", "sensor_01", "location_01");

        let query_string = measurement.into_query_string("soil_moisture_readings");

        assert_eq!(
            query_string,
            format!(
                "soil_moisture_readings,id={},sensor_id=sensor_01,location=location_01,unit=cb data=30.00000 {}",
                measurement.id, measurement.time
            )
        );
    }
}
