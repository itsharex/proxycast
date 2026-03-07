import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

export interface MediaPreferenceSectionModelOption {
  value: string;
  label: string;
}

export interface MediaPreferenceSectionProviderOption {
  value: string;
  label: string;
}

interface MediaPreferenceSectionProps {
  title: string;
  description: string;
  providerLabel: string;
  providerValue: string;
  providerAutoLabel: string;
  onProviderChange: (value: string) => void;
  providers: MediaPreferenceSectionProviderOption[];
  providerUnavailableLabel?: string;
  modelLabel: string;
  modelValue: string;
  modelAutoLabel: string;
  onModelChange: (value: string) => void;
  models: MediaPreferenceSectionModelOption[];
  modelUnavailableLabel?: string;
  modelHint: string;
  allowFallback: boolean;
  onAllowFallbackChange: (value: boolean) => void;
  fallbackTitle: string;
  fallbackDescription: string;
  emptyHint?: string;
  disabled?: boolean;
  modelDisabled?: boolean;
  resetLabel?: string;
  onReset?: () => void;
  resetDisabled?: boolean;
}

export function MediaPreferenceSection({
  title,
  description,
  providerLabel,
  providerValue,
  providerAutoLabel,
  onProviderChange,
  providers,
  providerUnavailableLabel,
  modelLabel,
  modelValue,
  modelAutoLabel,
  onModelChange,
  models,
  modelUnavailableLabel,
  modelHint,
  allowFallback,
  onAllowFallbackChange,
  fallbackTitle,
  fallbackDescription,
  emptyHint,
  disabled = false,
  modelDisabled = false,
  resetLabel,
  onReset,
  resetDisabled = false,
}: MediaPreferenceSectionProps) {
  return (
    <div className="space-y-4 rounded-lg border p-4">
      <div className="flex items-start justify-between gap-3">
        <div>
          <h3 className="text-sm font-medium">{title}</h3>
          <p className="mt-1 text-xs text-muted-foreground">{description}</p>
        </div>
        {onReset ? (
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={onReset}
            disabled={disabled || resetDisabled}
          >
            {resetLabel ?? "恢复默认"}
          </Button>
        ) : null}
      </div>

      <div className="space-y-2">
        <Label>{providerLabel}</Label>
        <Select
          value={providerValue}
          onValueChange={onProviderChange}
          disabled={disabled}
        >
          <SelectTrigger>
            <SelectValue placeholder={providerLabel} />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="__auto__">{providerAutoLabel}</SelectItem>
            {providerUnavailableLabel ? (
              <SelectItem value={providerValue}>
                {providerUnavailableLabel}
              </SelectItem>
            ) : null}
            {providers.map((provider) => (
              <SelectItem key={provider.value} value={provider.value}>
                {provider.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        {emptyHint ? (
          <p className="text-xs text-muted-foreground">{emptyHint}</p>
        ) : null}
      </div>

      <div className="space-y-2">
        <Label>{modelLabel}</Label>
        <Select
          value={modelValue}
          onValueChange={onModelChange}
          disabled={disabled || modelDisabled}
        >
          <SelectTrigger>
            <SelectValue placeholder={modelLabel} />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="__auto__">{modelAutoLabel}</SelectItem>
            {modelUnavailableLabel ? (
              <SelectItem value={modelValue}>
                {modelUnavailableLabel}
              </SelectItem>
            ) : null}
            {models.map((model) => (
              <SelectItem key={model.value} value={model.value}>
                {model.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <p className="text-xs text-muted-foreground">{modelHint}</p>
      </div>

      <div className="flex items-center justify-between rounded-lg border px-3 py-3">
        <div className="space-y-1">
          <Label className="text-sm">{fallbackTitle}</Label>
          <p className="text-xs text-muted-foreground">{fallbackDescription}</p>
        </div>
        <Switch
          checked={allowFallback}
          onCheckedChange={onAllowFallbackChange}
        />
      </div>
    </div>
  );
}

export default MediaPreferenceSection;
