import React, { type ErrorInfo } from "react";
import { AlertTriangle } from "lucide-react";
import { Button } from "@/components/ui/button";
import { reportFrontendError } from "@/lib/crashReporting";

interface WorkbenchCreateContentDialogBoundaryProps {
  children: React.ReactNode;
  open: boolean;
  step: "mode" | "intent";
  mode: string;
}

interface WorkbenchCreateContentDialogBoundaryState {
  error: Error | null;
}

export class WorkbenchCreateContentDialogBoundary extends React.Component<
  WorkbenchCreateContentDialogBoundaryProps,
  WorkbenchCreateContentDialogBoundaryState
> {
  state: WorkbenchCreateContentDialogBoundaryState = {
    error: null,
  };

  static getDerivedStateFromError(error: Error) {
    return { error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo): void {
    const context = {
      step: this.props.step,
      mode: this.props.mode,
      componentStack: errorInfo.componentStack,
    };
    console.error("[WorkbenchCreateContentDialogBoundary] 捕获异常:", error, {
      ...context,
    });
    void reportFrontendError(error, {
      component: "WorkbenchCreateContentDialogBoundary",
      workflow_step: `workspace_creation_${this.props.step}`,
      creation_mode: this.props.mode,
      context,
    });
  }

  componentDidUpdate(
    prevProps: WorkbenchCreateContentDialogBoundaryProps,
  ): void {
    if (
      this.state.error &&
      (!this.props.open ||
        prevProps.step !== this.props.step ||
        prevProps.mode !== this.props.mode)
    ) {
      this.setState({ error: null });
    }
  }

  private handleRetry = () => {
    this.setState({ error: null });
  };

  render() {
    const { children, open, mode, step } = this.props;
    const { error } = this.state;

    if (!error) {
      return children;
    }

    if (!open) {
      return null;
    }

    return (
      <div className="fixed inset-0 z-[80] flex items-center justify-center bg-black/40 p-4">
        <div className="w-full max-w-sm rounded-lg border bg-background p-4 shadow-lg">
          <div className="flex items-start gap-3">
            <AlertTriangle className="h-5 w-5 text-destructive mt-0.5" />
            <div className="space-y-2">
              <p className="text-sm font-medium">创作表单加载失败</p>
              <p className="text-xs text-muted-foreground">
                请切换模式后重试。当前模式：{mode}，步骤：{step}
              </p>
              <Button size="sm" onClick={this.handleRetry}>
                重试
              </Button>
            </div>
          </div>
        </div>
      </div>
    );
  }
}

export default WorkbenchCreateContentDialogBoundary;
