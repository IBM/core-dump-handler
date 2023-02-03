{{/* vim: set filetype=mustache: */}}
{{/*
Expand the name of the chart.
*/}}
{{- define "core-dump-handler.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "core-dump-handler.fullname" -}}
{{- if .Values.fullnameOverride -}}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" -}}
{{- else -}}
{{- $name := default .Chart.Name .Values.nameOverride -}}
{{- if contains $name .Release.Name -}}
{{- .Release.Name | trunc 63 | trimSuffix "-" -}}
{{- else -}}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" -}}
{{- end -}}
{{- end -}}
{{- end -}}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "core-dump-handler.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{/*
Common labels
*/}}
{{- define "core-dump-handler.labels" -}}
helm.sh/chart: {{ include "core-dump-handler.chart" . }}
{{ include "core-dump-handler.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end -}}

{{/*
Selector labels
*/}}
{{- define "core-dump-handler.selectorLabels" -}}
app.kubernetes.io/name: {{ include "core-dump-handler.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end -}}

{{/*
Create the name of the service account to use
*/}}
{{- define "core-dump-handler.serviceAccountName" -}}
{{- if .Values.serviceAccount.create -}}
    {{ default (include "core-dump-handler.fullname" .) .Values.serviceAccount.name }}
{{- else -}}
    {{ default "default" .Values.serviceAccount.name }}
{{- end -}}
{{- end -}}

{{/*
Render values given either as string or yaml structure.
Basically copied from https://github.com/bitnami/charts/blob/master/bitnami/common/templates/_tplvalues.tpl
*/}}
{{- define "core-dump-handler.tplvalues.render" -}}
    {{- if typeIs "string" .value }}
        {{- tpl .value .context }}
    {{- else }}
        {{- tpl (.value | toYaml) .context }}
    {{- end }}
{{- end -}}

{{- define "core-dump-handler.daemonset.container.volumeMounts" -}}
- name: host-volume
  mountPath:  {{ .Values.daemonset.hostDirectory }}
  mountPropagation: Bidirectional
- name: core-volume
  mountPath:  {{ .Values.daemonset.coreDirectory }}
  mountPropagation: Bidirectional
{{- if .Values.composer.coreEvents }}
- name: event-volume
  mountPath:  {{ .Values.daemonset.eventDirectory }}
  mountPropagation: Bidirectional
{{- end }}
{{- if .Values.daemonset.mountContainerRuntimeEndpoint }}
- mountPath: {{ .Values.daemonset.hostContainerRuntimeEndpoint }}
  name: container-runtime
{{- end }}
{{- end -}}
