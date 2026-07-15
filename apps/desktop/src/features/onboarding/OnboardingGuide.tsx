export function OnboardingGuide() {
  return <section aria-labelledby="onboarding-title">
    <h2 id="onboarding-title">开始使用</h2>
    <ol>
      <li>提示词库与备份保存在此设备的数据目录，核心管理无需联网。</li>
      <li>先创建或导入提示词，在收件箱中补齐元数据后再发布。</li>
      <li>定期在“备份与恢复”中创建备份；恢复前先检查内容。</li>
      <li>AI 密钥仅应通过系统凭据存储设置；MCP 仅能读取内容或创建收件箱草稿。</li>
    </ol>
  </section>;
}
