import type { ApplicationStatus, DiagnosticsStatus, ImportJobSummary } from "@prompt-hub/contracts";

export function DiagnosticsPanel({ status, importJobs, diagnostics, rebuildSearchIndex }: { status: ApplicationStatus | null; importJobs: ImportJobSummary[] | null; diagnostics: DiagnosticsStatus | null; rebuildSearchIndex?: () => Promise<void> }) {
  return <section aria-labelledby="diagnostics-title">
    <h2 id="diagnostics-title">诊断信息</h2>
    {status ? <dl>
      <dt>应用版本</dt><dd>{status.appVersion}</dd>
      <dt>数据库架构</dt><dd>{status.databaseSchemaVersion}</dd>
      <dt>离线可用</dt><dd>{status.offlineCapable ? "是" : "否"}</dd>
      <dt>日志隐私</dt><dd>诊断不显示提示词正文、密钥或授权头。</dd>
    </dl> : <p>正在读取本地诊断信息…</p>}
    {diagnostics ? <dl>
      <dt>数据库</dt><dd>{diagnostics.databaseAvailable ? "可用" : "不可用"}</dd>
      <dt>搜索索引</dt><dd>{diagnostics.searchIndexConsistent ? "一致" : "需重建"}</dd>
      <dt>MCP 数据库</dt><dd>{diagnostics.mcpDatabaseAvailable ? "可用" : "不可用"}</dd>
      {!diagnostics.searchIndexConsistent && rebuildSearchIndex ? <button onClick={() => void rebuildSearchIndex()} type="button">重建搜索索引</button> : null}
    </dl> : <p>正在读取健康状态…</p>}
    <h3>最近导入任务</h3>
    {importJobs === null ? <p>正在读取导入任务状态…</p> : importJobs.length === 0 ? <p>尚无导入任务。</p> : <ul aria-label="最近导入任务">{importJobs.map((job) => <li key={job.id}>{job.sourceKind}：{job.status}，新增 {job.imported}，重复 {job.skippedDuplicates}，失败 {job.failed}。</li>)}</ul>}
  </section>;
}
