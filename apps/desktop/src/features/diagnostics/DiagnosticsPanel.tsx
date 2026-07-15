import type { ApplicationStatus } from "@prompt-hub/contracts";

export function DiagnosticsPanel({ status }: { status: ApplicationStatus | null }) {
  return <section aria-labelledby="diagnostics-title">
    <h2 id="diagnostics-title">诊断信息</h2>
    {status ? <dl>
      <dt>应用版本</dt><dd>{status.appVersion}</dd>
      <dt>数据库架构</dt><dd>{status.databaseSchemaVersion}</dd>
      <dt>离线可用</dt><dd>{status.offlineCapable ? "是" : "否"}</dd>
      <dt>日志隐私</dt><dd>诊断不显示提示词正文、密钥或授权头。</dd>
    </dl> : <p>正在读取本地诊断信息…</p>}
  </section>;
}
