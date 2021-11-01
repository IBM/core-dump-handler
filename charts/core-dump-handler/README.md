# Core Dump Handler

This helm chart is designed to deploy functionality that automatically saves core dumps from any public cloud kubernetes service provider or [RedHat OpenShift Kubernetes Service](https://cloud.ibm.com/kubernetes/catalog/create?platformType=openshift) to an S3 compatible storage service.

## Prerequisites

The [Helm](https://helm.sh/) cli to run the chart

An [S3](https://en.wikipedia.org/wiki/Amazon_S3) compatible object storage solution such as [IBM Cloud Object Storage](https://cloud.ibm.com/objectstorage/create)

A [CRIO](https://cri-o.io/) compatible container runtime on the kubernetes hosts. If you service provider uses something else we will willingly recieve patches to support them.

## Installing the Chart

```
git clone https://github.com/IBM/core-dump-handler
cd core-dump-handler/charts/core-dump-handler
helm install core-dump-handler . --create-namespace --namespace observe \
--set daemonset.s3AccessKey=XXX --set daemonset.s3Secret=XXX \
--set daemonset.s3BucketName=XXX --set daemonset.s3Region=XXX
```

Where the `--set` options are configuration for your S3 compatible provider
Details for [IBM Cloud are available](https://cloud.ibm.com/docs/cloud-object-storage?topic=cloud-object-storage-uhc-hmac-credentials-main)

### OpenShift

As the agent runs in privileged mode the following command is needed on OpenShift.
`-z` is the service account name and `-n` is the namespace.
```
oc adm policy add-scc-to-user privileged -z core-dump-admin -n observe
```
Some OpenShift services such as OpenShift on IBM Cloud run on RHEL7 if that's the case then add the folowing option to the helm command or update the values.yaml.
This will be apparent if you see errors relating to glibc in the composer.log in the install folder of the agent. [See Troubleshooting below](#troubleshooting)
```
--set daemonset.vendor=rhel7
```

### Verifying the Chart Installation

Run a crashing container - this container writes a value to a null pointer

1. kubectl run -i -t segfaulter --image=quay.io/icdh/segfaulter --restart=Never

2. Validate the core dump has been uploaded to your object store instance.

     
## Additional Parameters for Public Kubernetes Services 

|Provider  |Product  |Version  |Additional Params  |
|---|---|---|---|
|AWS|EKS|1.21|--set daemonset.includeCrioExe=true|
|Digital Ocean|K8S|1.21.5-do.0|--set daemonset.DeployCrioConfig=true --set daemonset.composerCrioImageCmd="images"|
|Google|GKE|1.20.9-gke.1001|[Ubuntu containerd image](https://cloud.google.com/kubernetes-engine/docs/concepts/node-images#ubuntu-variants) **must** be used for the worker nodes. No additional params required.|
|IBM|IKS|1.19,1.20|  |
|IBM|ROKS|4.6|Must enable privileged policy [See OpenShift Section]("#openshift)|
|Microsoft|AKS|1.19|  |

### Environment Variables

The agent pod has the following environment variables:

* COMP_LOG_LEVEL - The log level configuration passed to the composer

    Valid values: Debug, Info, Warn, Error
* COMP_IGNORE_CRIO - Defines if the composer should get additional container JSON from crictl

    false (Default): The composer will generate the additional JSON files.
    
    true: The composer will only collect the core dump and save the core parameters as an additional JSON
* COMP_CRIO_IMAGE_CMD - The command to use to get image information for the core dump.

    "img" (Default): This is the value most crictls expect.

    "images": Digital Ocean Required this value
* DEPLOY_CRIO_CONFIG - Defines whether the agent should deploy a crictl config to the host

    false (Default): Most hosts will already have crictl configuration so this is ignored

    true : will deploy a default crio config that points to the sockets in /run
* HOST_DIR - The path on the host node that is used to copy files and deploy the composer.

    Defaults to /var/mnt/core-dump-handler as that is the only writable location on some providers.
* SUID_DUMPABLE - Sets the fs.suid_dumpable kernel tunable on the host. 

    Defaults to 2.
* DEPLOY_CRIO_EXE - Defines whether the agent should deploy a crictl client to the host

    false (Default): Most hosts will already have crictl installed on the node.

    true : will deploy v1.22 version of crictl
* S3_ACCESS_KEY - The S3 access key for the bucket that will be uploaded to
* S3_SECRET - The secret that is used along with the access key
* S3_BUCKET_NAME - The name of the bucket to upload files too
* S3_REGION - The region configuration for the bucket
* VENDOR - Some older hosts may require targeted builds for the composer.

    default(Default) - A RHEL8 build

    rhel7 - A RHEL7 Build
* INTERVAL - The amount of time in milliseconds between each check of the core dump folder for files to upload.
* SCHEDULE - A CRON formatted string [See cron library](https://github.com/mvniekerk/tokio-cron-scheduler#usage).

### Secrets

The following secrets are configurable and map to the corresponding environment variables

* s3config

    key: s3AccessKey

    key: s3Secret

    key: s3BucketName

    key: s3Region

### Values

General
* storage: The size of the storage for the cores (Default 1Gi)
* storageClass: The storage class for volume (Default hostclass)

Image 
* request_mem: The request memory for the agent pod (Default "64Mi")
* request_cpu: The request cpu for the agent pod (Default "250m")
* limit_mem: The limit memory setting for the agent (Default "128Mi")
* limit_cpu: The limit cpu setting for the agent (Default "500m")

Daemonset
* hostDirectory: Maps to the HOST_DIR environment variable (Default "/var/mnt/core-dump-handler")
* composerLogLevel: The log level for the composer (Default "Warn")
* suidDumpable: Maps to the SUID_DUMPABLE environment variable (Default 2)
* vendor: Maps to the VENDOR enviroment variable (Default default) 
* interval: Maps to the INTERVAL enviroment variable (Default 60000)
* schedule: Maps to the INTERVAL enviroment variable (Default "")
* composerIgnoreCrio: Maps to the COMP_IGNORE_CRIO enviroment variable  (Default false)
* composerCrioImageCmd: Maps to the COMP_CRIO_IMAGE_CMD enviroment variable (Default "img")
* DeployCrioConfig:  Maps to the DEPLOY_CRIO_CONFIG enviroment variable (Default false)
* includeCrioExe: Maps to the DEPLOY_CRIO_EXE enviroment variable (Default false)
* manageStoreSecret: Defines if the chart will be responsible for creating the S3 environment variables.

Set to false if you are using an external secrets managment system (Default true)
* s3AccessKey : Maps to the S3_ACCESS_KEY enviroment variable
* s3Secret : Maps to the S3_SECRET enviroment variable
* s3BucketName : Maps to the S3_BUCKET_NAME enviroment variable
* 3Region : Maps to the S3_REGION enviroment variable
* extraEnvVars: Option for passing additional configuration to the agent such as endpoint properties.
