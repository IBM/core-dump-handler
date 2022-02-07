use advisory_lock::{AdvisoryFileLock, FileLockMode};
use cloudevents::binding::reqwest::RequestBuilderExt;
use cloudevents::{Event, EventBuilder, EventBuilderV10};
use serde::Serialize;
use serde_json::Value;
use std::fs::File;
use uuid::Uuid;

#[derive(Serialize)]
pub struct CoreEventManager {
    filelocation: String,
    event: Value,
    hostname: String,
}

impl CoreEventManager {
    pub fn new(filelocation: String) -> Result<CoreEventManager, String> {
        let file = File::open(&filelocation).unwrap();
        match file.try_lock(FileLockMode::Shared) {
            Ok(_) => { /* If we can lock then we are ok */ }
            Err(e) => {
                return Err(format!("File locked {}", e));
            }
        }
        let event: Value = serde_json::from_reader(file).unwrap();
        let hostname = event["hostname"].as_str().unwrap_or_default().to_string();
        Ok(CoreEventManager {
            filelocation,
            event,
            hostname,
        })
    }

    pub async fn fire_created_event(&self, target: String) -> Result<Event, String> {
        let event = match EventBuilderV10::new()
            .id(&Uuid::new_v4().to_hyphenated().to_string())
            .ty("info.coredump.created")
            .source(format!("http://{}", self.hostname))
            .data(
                "application/json",
                serde_json::to_value(self).unwrap_or_default(),
            )
            .build()
        {
            Ok(v) => v,
            Err(e) => {
                return Err(format!("Failed to build event {}", e));
            }
        };
        let retval = event.clone();
        let client = match reqwest::Client::new().post(&target).event(event) {
            Ok(v) => v,
            Err(e) => {
                return Err(format!("Failed to create request client {}", e));
            }
        };

        let response = match client
            .header("Access-Control-Allow-Origin", "*")
            .send()
            .await
        {
            Ok(v) => v,
            Err(e) => {
                return Err(format!("Failed to send request {}", e));
            }
        };
        if response.status() == 200 {
            Ok(retval)
        } else {
            Err(format!("Response was not a 200 {}", response.status()))
        }
    }

    // pub fn fire_upload_event() {}
}

#[cfg(test)]
mod tests {
    use crate::events::CoreEventManager;
    use cloudevents::AttributesReader;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn setup() -> CoreEventManager {
        let dir = std::env::current_dir()
            .unwrap()
            .to_string_lossy()
            .to_string();
        println!("{}", dir);
        let file = format!(
            "{}/mocks/c8c89865-57ab-4530-abc2-96347106da8a-event.json",
            dir
        );
        CoreEventManager::new(file).unwrap()
    }
    #[tokio::test]
    async fn create_fire_event() {
        let mock_server = MockServer::start().await;

        let eventmgr = setup();

        Mock::given(method("POST"))
            .and(path("/"))
            .respond_with(ResponseTemplate::new(200))
            // Mounting the mock on the mock server - it's now effective!
            .mount(&mock_server)
            .await;

        let evt = eventmgr
            .fire_created_event(format!("{}/", &mock_server.uri()))
            .await
            .unwrap();
        assert_eq!(evt.source(), "http://hostname");
        let data: serde_json::Value = evt.data().unwrap().clone().try_into().unwrap();
        assert_eq!(
            data["event"]["labels"]["info.coredump.repo"],
            "core-dump-handler".to_string()
        );
        assert_eq!(
            data["event"]["labels"]["info.coredump.owner"],
            "no9".to_string()
        )
    }
}
