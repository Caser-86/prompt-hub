import { Component, type ReactNode } from "react";

type ErrorBoundaryProps = {
  children: ReactNode;
};

type ErrorBoundaryState = {
  hasError: boolean;
};

export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  public state: ErrorBoundaryState = { hasError: false };

  public static getDerivedStateFromError(): ErrorBoundaryState {
    return { hasError: true };
  }

  public render() {
    if (this.state.hasError) {
      return (
        <main className="application-error">
          <section aria-label="应用错误" role="alert">
            <h1>应用遇到无法显示的内容</h1>
            <p>当前数据未被修改。请重试；若问题持续，请查看诊断信息。</p>
            <button onClick={() => this.setState({ hasError: false })} type="button">
              重试
            </button>
          </section>
        </main>
      );
    }

    return this.props.children;
  }
}
