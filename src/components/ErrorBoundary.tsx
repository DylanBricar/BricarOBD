import React, { ReactNode } from "react";
import i18n from "i18next";

interface Props {
  children: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

export default class ErrorBoundary extends React.Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error) {
    console.error("[BricarOBD] Error caught by boundary:", error);
  }

  handleReload = () => {
    window.location.reload();
  };

  render() {
    if (this.state.hasError) {
      const title = i18n.t("common.errorBoundaryTitle");
      const message = i18n.t("common.errorBoundaryMessage");
      const buttonText = i18n.t("common.errorBoundaryReload");

      return (
        <div className="flex h-screen w-screen items-center justify-center bg-obd-bg">
          <div className="w-full max-w-md px-6 py-8">
            <div className="glass-card p-6 rounded-lg border border-obd-border/30">
              <h1 className="text-2xl font-bold text-obd-text mb-4">{title}</h1>
              <p className="text-obd-text/80 mb-6">{message}</p>
              {this.state.error && (
                <div className="mb-6 p-4 bg-obd-surface/50 rounded border border-obd-danger/30">
                  <p className="text-xs text-obd-danger/90 font-mono break-words">
                    {this.state.error.message}
                  </p>
                </div>
              )}
              <button
                onClick={this.handleReload}
                className="w-full px-4 py-2 bg-obd-accent hover:bg-obd-accent/90 text-white rounded font-medium transition-colors"
              >
                {buttonText}
              </button>
            </div>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}
