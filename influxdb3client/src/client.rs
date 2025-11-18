use log::error;
use reqwest::Client;
use tokio::time::{Duration, sleep};

use crate::SoilMoistureMeasurement;

// Represents a client that can push data into an influxdb3-core database
pub struct InfluxDB3Client {
    client: Client,
    url: String,
    token: String,
    table: String,
}

impl InfluxDB3Client {
    pub fn new<S: Into<String>>(url: S, token: S, table: S) -> Self {
        InfluxDB3Client {
            client: Client::new(),
            url: url.into(),
            token: token.into(),
            table: table.into(),
        }
    }

    // write a measurement to the database
    pub async fn write_query(
        &self,
        measurement: &SoilMoistureMeasurement,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let query = measurement.into_query_string(&self.table);
        let auth_header = format!("Bearer {}", &self.token);

        // send query
        let res = self
            .client
            .post(&self.url)
            .header("Authorization", auth_header)
            .body(query)
            .send()
            .await?;

        Ok(res.status().is_success())
    }

    // if the query fails, this method retries the operation until max_retires has been reached
    pub async fn write_query_with_retry(
        &self,
        measurement: &SoilMoistureMeasurement,
        max_retries: u8,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let mut delay = Duration::from_secs(1);
        let mut retries = 1;

        loop {
            match self.write_query(measurement).await {
                Ok(_) => return Ok(true),
                Err(err) => {
                    retries += 1;

                    if retries >= max_retries {
                        error!("Max retries reached, db write failed: {}", err);
                        return Err(err);
                    }
                    error!(
                        "Error writing to db (attempt: {}) retrying in {:?} {}",
                        retries, delay, err
                    );

                    sleep(delay).await;
                    delay = std::cmp::min(delay * 2, Duration::from_secs(32));
                }
            }
        }
    }
}
