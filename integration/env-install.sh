#! /bin/bash

cd ../
export $(grep -v '^#' .env | xargs)

cd ./charts/core-dump-handler

helm install core-dump-handler . --create-namespace --namespace observe \
--set daemonset.s3BucketName=${S3_BUCKET_NAME} --set daemonset.s3Region=${S3_REGION} \
--set daemonset.s3AccessKey=${S3_ACCESS_KEY} --set daemonset.s3Secret=${S3_SECRET}
## Poll until pod is up just give kube a second to recognise the helm chart has been applied
sleep 1
# while [[ $(kubectl get pods -n observe -l name=core-dump-ds -o 'jsonpath={..status.conditions[?(@.type=="Ready")].status}') != "True" ]]; 
while [[ $(kubectl get pods -n observe -l name=core-dump-ds -o json | jq -r '.items[].status.conditions[].status | select(.=="False")') == *"False"* ]];
do
    kubectl get pods -n observe -l name=core-dump-ds -o json | jq -r '.items[].status.conditions[]'
    echo "When all items status are 'True' the core pods are set up" && sleep 1;
done

echo "Core dump pod is ready - starting crash test"