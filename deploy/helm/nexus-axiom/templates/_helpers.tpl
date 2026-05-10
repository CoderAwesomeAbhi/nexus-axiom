{{- define "nexus-axiom.fullname" -}}
{{- printf "%s" .Release.Name | trunc 63 | trimSuffix "-" }}
{{- end }}

{{- define "nexus-axiom.labels" -}}
helm.sh/chart: {{ .Chart.Name }}-{{ .Chart.Version }}
{{ include "nexus-axiom.selectorLabels" . }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{- define "nexus-axiom.selectorLabels" -}}
app.kubernetes.io/name: nexus-axiom
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}
