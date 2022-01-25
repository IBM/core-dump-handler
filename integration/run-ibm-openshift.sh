#! /bin/bash

cd ../
export $(grep -v '^#' .env | xargs)

cd ./charts/core-dump-handler

helm install core-dump-handler . --create-namespace --namespace observe \
--set daemonset.s3BucketName=${S3_BUCKET_NAME} --set daemonset.s3Region=${S3_REGION} \
--set daemonset.s3AccessKey=${S3_ACCESS_KEY} --set daemonset.s3Secret=${S3_SECRET} \
--values values.roks.yaml
## Poll until pod is up
sleep 1
# while [[ $(kubectl get pods -n observe -l name=core-dump-ds -o 'jsonpath={..status.conditions[?(@.type=="Ready")].status}') != "True" ]]; 
while [[ $(kubectl get pods -n observe -l name=core-dump-ds -o json | jq -r '.items[].status.conditions[].status | select(.=="False")') == *"False"* ]];
do
    kubectl get pods -n observe -l name=core-dump-ds -o json | jq -r '.items[].status.conditions[]'
    echo "When all items status are 'True' the core pods are set up" && sleep 1;
done

sleep 1
echo "Core dump pod is ready - starting crash test"

kubectl run -it segfaulter --image=quay.io/icdh/segfaulter --restart=Never

kubectl delete pod segfaulter

mc alias set storage https://$S3_REGION $S3_ACCESS_KEY $S3_SECRET

while [[ $(mc find storage/${S3_BUCKET_NAME} --newer-than 1m) == "" ]]; 
do 
    echo "waiting for upload to complete" && sleep 1;
done

file_name=$(mc find storage/${S3_BUCKET_NAME} --newer-than 1m)
base_name=$(basename ${file_name})
rm -fr ./output
mkdir ./output 
cd ./output

echo "Copying $base_name"
mc cp $file_name .

unzip $base_name 

cleanup() {
    mc rm $file_name
    helm delete -n observe core-dump-handler
    exit 1
}

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

node_hostname=$(jq -r '.node_hostname' *-dump-info.json)

if [[ "$node_hostname" ]];
then
    echo -e "${GREEN}Success: Node Name Exists${NC}"
else
    echo -e "${RED}Failed${NC}"
    echo "Node Does NOT Name Exists ${node_hostname}"
    echo "Examine the output folder"
    cleanup
fi

log_file_count=$(wc -l < *.log)

# There seems to be a bug in ROKS 4.8 where this command only returns exactly half of the tail.
# This also seems to be the same when using the kubectl client.
if [[ "$log_file_count" == "500" ]];
then
    echo -e "${GREEN}Success: logfile contains 500 lines${NC}"
    if [[ "$log_file_count" == "250" ]];
    then
        echo -e "${GREEN}Success: logfile contains 250 lines${NC}"

    else
        echo -e "${RED}Failed${NC}"
        echo "Log file Does NOT contain 500 lines: Actual Count ${log_file_count}"
        echo "Examine the output folder"
        cleanup
    fi
fi

repoTags0=$(jq -r '.repoTags[0]' *0-image-info.json)
if [[ "$repoTags0" == "quay.io/icdh/segfaulter:latest" ]];
then
    echo -e "${GREEN}Success: Image successfully captured${NC}"
else
    echo -e "${RED}Failed${NC}"
    echo "Image NOT available ${repoTags0}"
    echo "Examine the output folder"
    cleanup
fi

file_count=$(ls | wc -l)

if [ $file_count == "8" ]
then
    echo -e "${GREEN}Success: Correct File Count${NC}"
    cd ..
    rm -fr output
else
    echo -e "${RED}Failed${NC}"
    echo "expected 8 files including the zip but found ${file_count}"
    echo "Examine the output folder"
    cleanup
fi
