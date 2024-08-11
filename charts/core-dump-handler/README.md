# Core Dump Handler

This helm chart is designed to deploy functionality that automatically saves core dumps from any public cloud kubernetes service provider or [RedHat OpenShift Kubernetes Service](https://cloud.ibm.com/kubernetes/catalog/create?platformType=openshift) to an S3 compatible storage service.

## Prerequisites

The [Helm](https://helm.sh/) cli to run the chart

An [S3 Protocol Compatible](https://en.wikipedia.org/wiki/Amazon_S3) object storage solution.

A [CRIO](https://cri-o.io/) compatible container runtime on the kubernetes hosts. If you service provider uses something else we will willingly recieve patches to support them.

## Installing the Chart

```
helm repo add core-dump-handler https://ibm.github.io/core-dump-handler/

helm install core-dump-handler . --create-namespace --namespace observe \
--set daemonset.s3AccessKey=XXX --set daemonset.s3Secret=XXX \
--set daemonset.s3BucketName=XXX --set daemonset.s3Region=XXX
```

Where the `--set` options are configuration for your S3 protocol compatible provider

For the following providers an additional option of values should be provided using the `--values` flag

e.g.
```
helm install core-dump-handler . --create-namespace --namespace observe \
--set daemonset.s3AccessKey=XXX --set daemonset.s3Secret=XXX \
--set daemonset.s3BucketName=XXX --set daemonset.s3Region=XXX \
--values values.aws.yaml
```

<table><thead><td>Provider</td><td>Product</td><td>Values</td></thead>
<tr>
    <td>AWS</td><td>EKS</td><td><a href="values.aws.yaml">values.aws.yaml</a></td>
</tr>
<tr>
    <td>AWS</td><td>EKS with IAM roles for service accounts</td><td><a href="values.aws.sts.yaml">values.aws.yaml</a></td>
</tr>
<tr>
    <td>AWS</td><td>EKS with Bottlerocket nodes</td><td><a href="values.aws.bottlerocket.yaml">values.bottlerocket.yaml</a></td>
</tr>
<tr>
    <td>AWS</td><td>ROSA</td><td><a href="values.openshift.yaml">values.openshift.yaml</a></td>
</tr>
<tr>
    <td>Digital Ocean</td><td>K8S</td><td><a href="values.do.yaml">values.do.yaml</a></td>
</tr>
<tr>
    <td>Google</td><td>GKE COS</td><td><a href="values.gke-cos.yaml">values.gke-cos.yaml</a></td>
</tr>
<tr>
    <td>IBM</td><td>IKS</td><td><a href="values.ibm.yaml">values.ibm.yaml</a></td>
</tr>
<tr>
    <td>IBM</td><td>ROKS</td><td><a href="values.roks.yaml">values.roks.yaml</a></td>
</tr>
<tr>
    <td>Microsoft</td><td>ARO</td><td><a href="values.openshift.yaml">values.openshift.yaml</a></td>
</tr>
<tr>
    <td>RedHat</td><td>On-Premises</td><td><a href="values.openshift.yaml">values.openshift.yaml</a></td>
</tr>
</table>

### Verifying the Chart Installation

Run a crashing container - this container writes a value to a null pointer

1. kubectl run -i -t segfaulter --image=quay.io/icdh/segfaulter --restart=Never

2. Validate the core dump has been uploaded to your object store instance.

### OpenShift

The agent runs in privileged mode you can enable to create a custom SCC along its service account during installation.
This configuration is catered for when you use the recommended values files `values.openshift.yaml` or `values.roks.yaml` but you may wish to either provide the config directly or apply the config using `oc`.

```
helm install core-dump-handler . --create-namespace --namespace observe \
...
--set serviceAccount.create=true \
--set scc.create=true
```

Manually, you can run this using `oc adm policy` where `-z` is the service account name and `-n` is the namespace.
```
oc adm policy add-scc-to-user privileged -z core-dump-admin -n observe
```

When running OpenShift on RHCOS (Red Hat CoreOS), you need to set different mount paths. A common writable path would be `/mnt/`, which you can control by setting:

```
helm install core-dump-handler . --create-namespace --namespace observe \
...
--set daemonset.hostDirectory=/mnt/core-dump-handler \
--set daemonset.coreDirectory=/mnt/core-dump-handler/cores
```

Some OpenShift services such as OpenShift on IBM Cloud run on RHEL7 if that's the case then add the folowing option to the helm command or update the values.yaml.

```
helm install core-dump-handler . --create-namespace --namespace observe \
--set daemonset.vendor=rhel7
```

You can make use of a more compact values.yaml during installation to override for the respective openshift values:
```
helm install core-dump-handler . --create-namespace --namespace observe --values values.openshift.yaml
```

### EKS setup for gitops pipelines (`eksctl` or similar)

Set up a service account with a role that has access to S3 bucket (in `cluster.yaml`):

```yaml
iam:
  withOIDC: true
  serviceAccounts:
    - metadata:
      name: core-dump-admin
      namespace: core-dump
    attachPolicyARNs:
      - arn:aws:iam::123456789011:policy/s3-write-policy
```

**Note**: here the namespace is `core-dump`, change it to the namespace where you installed the chart

Example S3 policy:

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": "s3:*",
      "Resource": [
        "arn:aws:s3:::my-core-dump-bucket",
        "arn:aws:s3:::my-core-dump-bucket/*"
      ]
    }
  ]
}
```

### EKS setup with IAM roles for service accounts

This allows core-dump-handler to automatically assume the correct role with permissions on the S3 bucket without providing fixed credentials in the secret.

See [this guide](https://docs.aws.amazon.com/eks/latest/userguide/iam-roles-for-service-accounts.html).

[Example of `values.yaml`](values.aws.sts.yaml)

### AWS Graviton and MUSL support

An MUSL based ARM build is available in https://quay.io/repository/icdh/core-dump-handler-musl 
Simply change the repository tag in the values.yaml
```
    repository: icdh/core-dump-handler-musl
```
Or set the value at the command prompt
```
helm --set image.repository=icdh/core-dump-handler-musl
```
### Environment Variables

The agent pod has the following environment variables and these are all set by the chart but included here for informational purposes:

* COMP_LOG_LEVEL - The log level configuration passed to the composer

    Valid values: Debug, Info, Warn, Error
* COMP_IGNORE_CRIO - Defines if the composer should get additional container JSON from crictl

    false (Default): The composer will generate the additional JSON files.

    true: The composer will only collect the core dump and save the core parameters as an additional JSON
* COMP_INCLUDE_PROC_INFO - Defines if the composer should get additional proc files from the container process. *Warning:* These files can contain sensitive information regarding the process, use with caution

    false (Default): The composer will not collect any pid related files

    true: The composer will copy 'auxv', 'cmdline', 'environ', 'maps' and 'status' for every container into the zip
* COMP_CRIO_IMAGE_CMD - The command to use to get image information for the core dump.

    "img" (Default): This is the value most crictls expect.
    "images": Digital Ocean, Newer OpenShift require this value

* COMP_TIMEOUT - The timeout for the composer in seconds. Defaults to 600.

    In testing ~ 3 mins per 512Mb so we have set it to 10 mins.

* COMP_COMPRESSION - Enable compression Default: true

    Given the amount of time compression there is an option to disable it.

* COMP_CORE_EVENTS - Enable the creation of a core event file Default: false

    Generates a file in a dedicated folder to be picked up by an external process.

* COMP_CORE_EVENT_DIR - The folder where the core dump event is saved.

* CRIO_ENDPOINT - The CRIO endpoint to use.

    "unix:///run/containerd/containerd.sock" (Default): This is the default for most containerd nodes
    "unix:///var/run/dockershim.sock": Should match most nodes that still use dockershim

* COMP_FILENAME_TEMPLATE - Defines the template that generates the filename using [tinytemplate](https://crates.io/crates/tinytemplate#quickstart) and the [params object](https://github.com/IBM/core-dump-handler/blob/main/core-dump-composer/src/config.rs#L29)

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
* S3_REGION - The region configuration for the bucket.

  When using an S3 compatible service you should enter the hostname of the service for this value.
  
  e.g. --set S3_REGION=host.mycloud.com 

  See https://github.com/IBM/core-dump-handler/issues/124 for further discussion. 
* VENDOR - Some older hosts may require targeted builds for the composer.

    default(Default) - A RHEL8 build

    rhel7 - A RHEL7 Build
* INTERVAL - The amount of time in milliseconds between each check of the core dump folder for files to upload.
* SCHEDULE - A CRON formatted string [See cron library](https://github.com/mvniekerk/tokio-cron-scheduler#usage).
* USE_INOTIFY - Set a listener for the coredump folder can be used in conjunction with SCHEDULE
* COMP_POD_SELECTOR_LABEL - Optional selector label to filter pods that have core dump collection enabled. Default (empty) disables filter and enables collection for all. E.g. when selector label is set as "my.org/batch-workload" only pods that have a label named "my.org/batch-workload" (any value) will be enabled for core dump collection.

### Secrets

The following secrets are configurable and map to the corresponding environment variables

* s3config

    key: s3AccessKey

    key: s3Secret

    key: s3BucketName

    key: s3Region

### Values

General
* priorityClassName: The priorityClass of the daemonset pods ( Default nil ) 
* storage: The size of the storage for the cores (Default 1Gi)
* storageClass: The storage class for volume (Default hostclass)

Image
* registry: image registry	(Default quay.io)
* repository: image repository (Default icdh/core-dump-handler)
* tag: image tag - immutable tags are recommended (Default 7.0.0)
* request_mem: The request memory for the agent pod (Default "64Mi")
* request_cpu: The request cpu for the agent pod (Default "250m")
* limit_mem: The limit memory setting for the agent (Default "128Mi")
* limit_cpu: The limit cpu setting for the agent (Default "500m")

Composer
* logLevel: The log level for the composer (Default "Warn")
* ignoreCrio: Maps to the COMP_IGNORE_CRIO enviroment variable  (Default false)
* includeProcInfo: Maps to the COMP_INCLUDE_PROC_INFO enviroment variable  (Default false)
* crioImageCmd: Maps to the COMP_CRIO_IMAGE_CMD enviroment variable (Default "img")
* timeout: Maps to the COMP_TIMEOUT environment variable ("Default 600)
* compression: Maps to the COMP_COMPRESSION environment variable (Default "true")
* coreEvents: Maps to the COMP_CORE_EVENTS envrironment variable (Default "false")
* filenameTemplate: Maps to COMP_FILENAME_TEMPLATE environment variable
    (Default {{uuid}}-dump-{{timestamp}}-{{hostname}}-{{exe_name}}-{{pid}}-{{signal}})

    Possible Values:

    limit_size - Core file size soft resource limit of crashing process"),

    exe_name - The process or thread's comm value, which typically is the
               same as the executable filename (without path prefix, and
               truncated to a maximum of 15 characters)

    pid - PID of dumped process, as seen in the PID namespace in which the process resides.",

    signal - Number of signal causing dump.

    timestamp - Time of dump, expressed as seconds since the Epoch.

    hostname - Same as nodename returned by uname(2)

    pathname - Pathname of executable, with slashes ('/') replaced by exclamation marks ('!'),

    uuid - a unique id generated for this core dump

    namespace - the namespace the pod is associated with.

* logLength: The amount of lines to take from the crashing pod. (Default 500)
* podSelectorLabel: Enable composer only if pod has label matching the specified selector. (Default "" matches all pods)

Daemonset
* podAnnotations: Annotations to be added to core-dump-handler pods (Default nil)
* hostDirectory: Maps to the HOST_DIR environment variable (Default "/var/mnt/core-dump-handler")
* coreDirectory: Maps to the CORE_DIR environment variable (Default "/var/mnt/core-dump-handler/cores")
* eventDirectory: Maps to the EVENT_DIR environment variable (Default "/var/mnt/core-dump-handler/events")
* suidDumpable: Maps to the SUID_DUMPABLE environment variable (Default 2)
* vendor: Maps to the VENDOR enviroment variable (Default default)
* interval: Maps to the INTERVAL enviroment variable (Default 60000)
* schedule: Maps to the SCHEDULE enviroment variable (Default "")
* useINotify: Maps to the USE_INOTIFY environment variable (Default false)
* DeployCrioConfig:  Maps to the DEPLOY_CRIO_CONFIG enviroment variable (Default false)
* includeCrioExe: Maps to the DEPLOY_CRIO_EXE enviroment variable (Default false)
* manageStoreSecret: Defines if the chart will be responsible for creating the S3 environment variables.

    Set to false if you are using an external secrets managment system (Default true)

* s3AccessKey : Maps to the S3_ACCESS_KEY enviroment variable
* s3Secret : Maps to the S3_SECRET enviroment variable
* s3BucketName : Maps to the S3_BUCKET_NAME enviroment variable
* 3Region : Maps to the S3_REGION enviroment variable
* extraEnvVars: Option for passing additional configuration to the agent such as endpoint properties.
* envFrom: Array of [EnvFromSource](https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.25/#envfromsource-v1-core) to inject into main container.
* sidecarContainers: Array of [Container](https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.25/#container-v1-core) to define as part of the pod.
* updateStrategy: [DaemonsetUpdateStrategy](https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.25/#daemonsetupdatestrategy-v1-apps) is a struct used to control the update strategy for the DaemonSet.

Service account:
* useToken: automatically create a service account token secret
* tokenSecretName: name of the service account token secret to create if `serviceAccount.useToken` is `true` (Default: "core-dump-service-account-token")
