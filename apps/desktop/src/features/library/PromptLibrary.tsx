import { useEffect, useState } from "react";

type PromptLibraryProps = {
  loadPrompts: () => Promise<unknown[]>;
};

export function PromptLibrary({ loadPrompts }: PromptLibraryProps) {
  const [prompts, setPrompts] = useState<unknown[] | null>(null);

  useEffect(() => {
    void loadPrompts().then(setPrompts);
  }, [loadPrompts]);

  return (
    <section aria-labelledby="library-title" className="prompt-library">
      <div className="feature-heading">
        <div>
          <p className="eyebrow">LOCAL LIBRARY</p>
          <h1 id="library-title">提示词库</h1>
        </div>
        <button type="button">创建提示词</button>
      </div>
      {prompts?.length === 0 ? <p>还没有提示词资产</p> : null}
    </section>
  );
}
