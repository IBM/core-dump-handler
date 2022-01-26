# FAQ

- [How should I integrate my own uploader?](#how-should-i-integrate-my-own-uploader)

- [Why is my core dump truncated?](#why-is-my-core-dump-truncated)

- [Why is my log file exactly half of my configured line count?](why-is-my-log-file-exactly-half-of-my-configured-line-count)

- [Can I force an upload?](#can-i-force-an-upload)

- [What are the storage configuration options?](#what-are-the-storage-configuration-options)

## How should I integrate my own uploader?

The core dump handler is designed to quickly move the cores *"off-box"* to an object storage environment with as much additional runtime information as possible.
In order to provide the following benefits:

- The developer who needs to debug the core doesn't require access to the kubernetes host environment and can access the crash as soon as it has happened.

- Cores will not use disk space by residing on the host longer than required.

- As Object Storage APIs have migrated to S3 as a defacto standard post processing services for scrubbers and indexing the data are easier to implement.

It's strongly recommened that you maintain the upload pattern of moving the cores off the machine but you may wish to move them to a none S3 compabible host.

This scenario is possible but the following aspects need consideration:

1. The upload functionality needs to be disabled by commenting out the `schedule` and `interval` properties and setting the `useINotify` to be `false`. See [main.rs](https://github.com/IBM/core-dump-handler/blob/main/core-dump-agent/src/main.rs#L153) for the details.

    e.g. [values.yaml](https://github.com/IBM/core-dump-handler/blob/main/charts/core-dump-handler/values.yaml#L30)
    ```yaml
    # interval: 60000
    # schedule: "* * * * * *"
    useINotify: false
    ```
    N.B. The interval and schedule **MUST** be removed as the JSON schema validation makes them mutually exclusive.

2. File Locks need to be honoured.

    The core dump composer uses [libc flock](https://man7.org/linux/man-pages/man2/flock.2.html) in the [advisory-lock-rs project](https://github.com/topecongiro/advisory-lock-rs/blob/master/src/unix.rs#L61) and the agent then trys to [obtain a shared lock](https://github.com/IBM/core-dump-handler/blob/main/core-dump-agent/src/main.rs#L296) to ensure that the file isn't still being written to when an upload is attempted.

    Your own uploader should respect these semantics.

3. Disable the S3 credentials validation by setting the `manageStoreSecret` in the values file to false.

    ```yaml
    manageStoreSecret: true
    ```

4. Your custom uploader should probably be configured and deployed outside of the core-dump-handler in order to facilitate upgrading the project in your environment. But if there is a hard requirement to integrate into this tool then please add a note to the [operator project](https://github.com/IBM/core-dump-operator/issues/1).

5. To assist in the configuration of the you custom container consider using the [pvc](https://github.com/IBM/core-dump-handler/blob/main/charts/core-dump-handler/templates/core-storage-pvc.yaml) and [pv](https://github.com/IBM/core-dump-handler/blob/main/charts/core-dump-handler/templates/core-storage-pv.yaml) configuration that is provided as part of this project but ensure that you change the names.

## Why is my core dump or information is truncated?

In some scenarios the core file can be truncated and/or the JSON info files may not be captured.
This is usually due to the default grace period during for [pod termination](https://kubernetes.io/docs/concepts/workloads/pods/pod-lifecycle/#pod-termination) being exceeded.
If this is a potential issue then set the `terminationGracePeriodSeconds` option in the Pod YAML to help mitigate this.

e.g. To change it to 120 seconds
```yaml
apiVersion: v1
kind: Pod
metadata:
  name: my-pod
spec:
  containers
  - name: my-container
    image: busybox
terminationGracePeriodSeconds: 120
```
Also see [Kubernetes best practices: terminating with grace](https://cloud.google.com/blog/products/containers-kubernetes/kubernetes-best-practices-terminating-with-grace)

## Why is my log file exactly half of my configured line count?

This appears to be a bug in some kubernetes services.
You should also notice that the command `kubectl logs --tail=500 YOUR_POD_ID` exhibits the same behaviour.
Current workaround is to double the line count on the configuration.


## Can I force an upload?

Some users run the agent in schedule mode to ensure that bandwidth during peak times isn't impacted. In this scenario you may wish to force an upload if there is a critical core that you need to get access to.

This can be achieved by logging into the agent container and executing the `sweep` command.

```
kubectl exec -it -n observe core-dump-handler-gcvtc -- /bin/bash
./core-dump-agent sweep
```

## What are the storage configuration options?

By default the upload to S3 compatible storage is configured using the storage parameters outlined in the install documents. However you may wish to integrate an external secrets management system to lay out your secrets outside of this helm chart.

In that case disable this chart from requiring the secrets by setting manageStoreSecret to false in the `values.yaml`.

```yaml
manageStoreSecret: false
```

Or by passing the following option when you deploy the chart: 
```
--set manageStoreSecret=false
```

Ensure that your secrets manager has layed out your secret as defined in the [secrets.yaml](https://github.com/IBM/core-dump-handler/blob/main/charts/core-dump-handler/templates/secrets.yaml) template.

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: s3config
type: Opaque
stringData:
  s3Secret: {{ .Values.daemonset.s3Secret }}
  s3AccessKey: {{ .Values.daemonset.s3AccessKey }}
  s3BucketName: {{ .Values.daemonset.s3BucketName }}
  s3Region: {{ .Values.daemonset.s3Region }}
```