use crate::config::CoreParams;
use advisory_lock::{AdvisoryFileLock, FileLockMode};
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use uuid::Uuid;

#[derive(Serialize)]
pub struct CoreEvent {
    image_list: Vec<String>,
    key: String,
    exe_path: String,
    labels: HashMap<String, String>,
    limit_size: String,
    exe_name: String,
    pid: String,
    signal: String,
    timestamp: String,
    hostname: String,
    namespace: Option<String>,
    uuid: Uuid,
}

impl CoreEvent {
    pub fn new_no_crio(core: CoreParams, zip_name: String) -> CoreEvent {
        let images: Vec<String> = vec![];
        let hm = HashMap::new();
        CoreEvent {
            image_list: images.to_vec(),
            key: zip_name,
            exe_path: core.pathname,
            labels: hm,
            limit_size: core.limit_size,
            exe_name: core.exe_name,
            pid: core.pid,
            signal: core.signal,
            timestamp: core.timestamp,
            hostname: core.hostname,
            namespace: core.namespace,
            uuid: core.uuid,
        }
    }
    pub fn new(
        core: CoreParams,
        zip_name: String,
        pod_info: Value,
        image_info: Vec<Value>,
    ) -> CoreEvent {
        let mut hm = HashMap::new();

        if let Some(labels) = pod_info["labels"].as_object() {
            for (name, label) in labels.iter() {
                if name.starts_with("info.coredump") {
                    hm.insert(
                        name.to_string(),
                        label.as_str().unwrap_or_default().to_string(),
                    );
                    println!("{label:?}");
                }
            }
        }

        let mut images: Vec<String> = vec![];

        for img in image_info {
            let img_digest = img["repoDigests"][0].as_str().unwrap_or_default();
            images.push(img_digest.to_string());
        }

        CoreEvent {
            image_list: images.to_vec(),
            key: zip_name,
            exe_path: core.pathname,
            labels: hm,
            limit_size: core.limit_size,
            exe_name: core.exe_name,
            pid: core.pid,
            signal: core.signal,
            timestamp: core.timestamp,
            hostname: core.hostname,
            namespace: core.namespace,
            uuid: core.uuid,
        }
    }

    pub fn write_event(&self, eventlocation: &str) -> Result<(), anyhow::Error> {
        let full_path = format!("{}/{}-event.json", eventlocation, self.uuid);
        let file = File::create(full_path)?;
        file.lock(FileLockMode::Exclusive)?;
        serde_json::to_writer(&file, &self)?;
        file.unlock()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::events::CoreEvent;
    use crate::events::CoreParams;
    use serde_json::json;
    use serde_json::Value;
    use std::fs;
    use std::path::Path;
    use uuid::Uuid;

    #[test]
    fn create_file_test() {
        let event = setup_with_labels();
        let dir = std::env::current_dir()
            .unwrap()
            .to_string_lossy()
            .to_string();
        event.write_event(&dir).unwrap();
        let full_path = format!("{}/{}-event.json", dir, event.uuid);

        assert!(Path::new(&full_path).exists());

        fs::remove_file(full_path).unwrap();
    }

    #[test]
    fn create_coreevent_without_labels_test() {
        let event = setup_without_labels();
        assert_eq!(event.labels.len(), 0);
    }

    #[test]
    fn create_coreevent_with_labels_test() {
        let event = setup_with_labels();
        assert_eq!(
            event.labels["info.coredump.repo"],
            "core-dump-handler".to_string()
        );
        assert_eq!(event.labels["info.coredump.owner"], "no9".to_string())
    }

    #[test]
    fn create_coreevent_with_images_test() {
        let event = setup_with_labels();
        assert_eq!(event.image_list[0], "docker.io/number9/example-crashing-nodejs-app@sha256:b8fea40ed9da77307702608d1602a812c5983e0ec0b788fc6298985a40be3800".to_string());
        assert_eq!(event.image_list[1], "icr.io/ibm/ibmcloud-object-storage-driver@sha256:c796a4c693b4b7bf366c89208e96648d082836ebcb3bd03d8b63aca6883a69b0".to_string());
    }

    fn setup_without_labels() -> CoreEvent {
        let zip_name = "afile.zip".to_string();
        let limit_size = "limit-size".to_string();
        let exe_name = "exe-name".to_string();
        let pid = "pid".to_string();
        let signal = "signal".to_string();
        let timestamp = "timestamp".to_string();
        let directory = "directory".to_string();
        let hostname = "hostname".to_string();
        let pathname = "pathname".to_string();
        let uuid = Uuid::new_v4();
        let podname = "podname".to_string();

        let params = CoreParams {
            limit_size,
            exe_name,
            pid,
            signal,
            timestamp,
            directory,
            hostname,
            pathname,
            namespace: None,
            uuid,
            podname: Some(podname),
        };
        let pod = json!(
           {
             "id": "51cd8bdaa13a65518e790d307359d33f9288fc82664879c609029b1a83862db6",
             "metadata": {
               "name": "crashing-app-699c49b4ff-86wrh",
               "uid": "0c65ce05-bd3a-4db2-ad79-131186dc2086",
               "namespace": "default",
               "attempt": 0
             },
             "state": "SANDBOX_READY",
             "createdAt": "1618746959894040481",
             "labels": {
               "app": "crashing-app",
               "io.kubernetes.pod.name": "crashing-app-699c49b4ff-86wrh",
               "io.kubernetes.pod.namespace": "default",
               "io.kubernetes.pod.uid": "0c65ce05-bd3a-4db2-ad79-131186dc2086",
               "pod-template-hash": "848dc79df4"
             },
             "annotations": {
               "kubernetes.io/config.seen": "2021-04-18T11:55:58.909472224Z",
               "kubernetes.io/config.source": "api",
               "kubernetes.io/psp": "ibm-privileged-psp"
             },
             "runtimeHandler": ""
           }
        );

        let image1 = json!({
          "id": "sha256:3b8adc6c30f4e7e4afb57daef9d1c8af783a4a647a4670780e9df085c0525efa",
          "repoTags": [
            "docker.io/number9/example-crashing-nodejs-app:latest"
          ],
          "repoDigests": [
            "docker.io/number9/example-crashing-nodejs-app@sha256:b8fea40ed9da77307702608d1602a812c5983e0ec0b788fc6298985a40be3800"
          ],
          "size": "338054458",
          "uid": null,
          "username": "node"
        });
        let image2 = json!({
          "id": "sha256:4b97dc265156e2bc2fb2567003489cbf2b7d1e538e6b15712a12668d6aaa00fd",
          "repoTags": [
            "icr.io/ibm/ibmcloud-object-storage-driver:1.8.16"
          ],
          "repoDigests": [
            "icr.io/ibm/ibmcloud-object-storage-driver@sha256:c796a4c693b4b7bf366c89208e96648d082836ebcb3bd03d8b63aca6883a69b0"
          ],
          "size": "103453889",
          "uid": null,
          "username": ""
        });
        let images: Vec<Value> = vec![image1, image2];

        return CoreEvent::new(params, zip_name, pod, images);
    }

    fn setup_with_labels() -> CoreEvent {
        let zip_name = "afile.zip".to_string();
        let limit_size = "limit-size".to_string();
        let exe_name = "exe-name".to_string();
        let pid = "pid".to_string();
        let signal = "signal".to_string();
        let timestamp = "timestamp".to_string();
        let directory = "directory".to_string();
        let hostname = "hostname".to_string();
        let pathname = "pathname".to_string();
        let uuid = Uuid::new_v4();
        let podname = "podname".to_string();

        let params = CoreParams {
            limit_size,
            exe_name,
            pid,
            signal,
            timestamp,
            directory,
            hostname,
            pathname,
            namespace: None,
            uuid,
            podname: Some(podname),
        };
        let image1 = json!({
          "id": "sha256:3b8adc6c30f4e7e4afb57daef9d1c8af783a4a647a4670780e9df085c0525efa",
          "repoTags": [
            "docker.io/number9/example-crashing-nodejs-app:latest"
          ],
          "repoDigests": [
            "docker.io/number9/example-crashing-nodejs-app@sha256:b8fea40ed9da77307702608d1602a812c5983e0ec0b788fc6298985a40be3800"
          ],
          "size": "338054458",
          "uid": null,
          "username": "node"
        });
        let image2 = json!({
          "id": "sha256:4b97dc265156e2bc2fb2567003489cbf2b7d1e538e6b15712a12668d6aaa00fd",
          "repoTags": [
            "icr.io/ibm/ibmcloud-object-storage-driver:1.8.16"
          ],
          "repoDigests": [
            "icr.io/ibm/ibmcloud-object-storage-driver@sha256:c796a4c693b4b7bf366c89208e96648d082836ebcb3bd03d8b63aca6883a69b0"
          ],
          "size": "103453889",
          "uid": null,
          "username": ""
        });
        let images: Vec<Value> = vec![image1, image2];

        let pod = json!(
           {
             "id": "51cd8bdaa13a65518e790d307359d33f9288fc82664879c609029b1a83862db6",
             "metadata": {
               "name": "crashing-app-699c49b4ff-86wrh",
               "uid": "0c65ce05-bd3a-4db2-ad79-131186dc2086",
               "namespace": "default",
               "attempt": 0
             },
             "state": "SANDBOX_READY",
             "createdAt": "1618746959894040481",
             "labels": {
               "app": "crashing-app",
               "io.kubernetes.pod.name": "crashing-app-699c49b4ff-86wrh",
               "io.kubernetes.pod.namespace": "default",
               "io.kubernetes.pod.uid": "0c65ce05-bd3a-4db2-ad79-131186dc2086",
               "info.coredump.owner": "no9",
               "info.coredump.repo": "core-dump-handler",
               "pod-template-hash": "848dc79df4"
             },
             "annotations": {
               "kubernetes.io/config.seen": "2021-04-18T11:55:58.909472224Z",
               "kubernetes.io/config.source": "api",
               "kubernetes.io/psp": "ibm-privileged-psp"
             },
             "runtimeHandler": ""
           }
        );

        return CoreEvent::new(params, zip_name, pod, images);
    }
}
