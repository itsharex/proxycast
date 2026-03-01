/**
 * 初次安装引导 - 插件选择
 */

import styled from "styled-components";
import { Checkbox } from "@/components/ui/checkbox";
import { onboardingPlugins } from "../constants";

const Container = styled.div`
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 32px 24px;
`;

const Title = styled.h2`
  font-size: 24px;
  font-weight: 600;
  color: hsl(var(--foreground));
  margin-bottom: 8px;
  text-align: center;
`;

const Subtitle = styled.p`
  font-size: 14px;
  color: hsl(var(--muted-foreground));
  margin-bottom: 24px;
  text-align: center;
`;

const PluginList = styled.div`
  display: flex;
  flex-direction: column;
  gap: 12px;
  width: 100%;
  max-width: 500px;
`;

const PluginCard = styled.label<{ $selected?: boolean }>`
  display: flex;
  align-items: flex-start;
  gap: 16px;
  padding: 16px;
  border-radius: 12px;
  border: 1px solid
    ${({ $selected }) =>
      $selected ? "hsl(var(--primary))" : "hsl(var(--border))"};
  background: ${({ $selected }) =>
    $selected ? "hsl(var(--primary) / 0.05)" : "hsl(var(--card))"};
  cursor: pointer;
  transition: all 0.2s;

  &:hover {
    border-color: hsl(var(--primary) / 0.5);
  }
`;

const CheckboxWrapper = styled.div`
  padding-top: 2px;
`;

const IconWrapper = styled.div<{ $selected?: boolean }>`
  width: 40px;
  height: 40px;
  border-radius: 10px;
  background: ${({ $selected }) =>
    $selected ? "hsl(var(--primary))" : "hsl(var(--muted))"};
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: all 0.2s;

  svg {
    width: 20px;
    height: 20px;
    color: ${({ $selected }) =>
      $selected ? "hsl(var(--primary-foreground))" : "hsl(var(--foreground))"};
  }
`;

const PluginInfo = styled.div`
  flex: 1;
  min-width: 0;
`;

const PluginName = styled.div`
  font-size: 14px;
  font-weight: 600;
  color: hsl(var(--foreground));
  margin-bottom: 4px;
`;

const PluginDescription = styled.div`
  font-size: 12px;
  color: hsl(var(--muted-foreground));
  line-height: 1.5;
`;

const SelectAllRow = styled.div`
  display: flex;
  justify-content: flex-end;
  width: 100%;
  max-width: 500px;
  margin-bottom: 8px;
`;

const SelectAllButton = styled.button`
  font-size: 12px;
  color: hsl(var(--primary));
  background: none;
  border: none;
  cursor: pointer;
  padding: 4px 8px;

  &:hover {
    text-decoration: underline;
  }
`;

interface PluginSelectStepProps {
  selectedPlugins: string[];
  onSelectionChange: (plugins: string[]) => void;
}

export function PluginSelectStep({
  selectedPlugins,
  onSelectionChange,
}: PluginSelectStepProps) {
  const handleToggle = (pluginId: string) => {
    if (selectedPlugins.includes(pluginId)) {
      onSelectionChange(selectedPlugins.filter((id) => id !== pluginId));
    } else {
      onSelectionChange([...selectedPlugins, pluginId]);
    }
  };

  const handleSelectAll = () => {
    if (selectedPlugins.length === onboardingPlugins.length) {
      onSelectionChange([]);
    } else {
      onSelectionChange(onboardingPlugins.map((p) => p.id));
    }
  };

  const isAllSelected = selectedPlugins.length === onboardingPlugins.length;

  return (
    <Container>
      <Title>选择要安装的插件</Title>
      <Subtitle>您可以根据需要选择插件，或稍后在插件中心安装</Subtitle>

      <SelectAllRow>
        <SelectAllButton onClick={handleSelectAll}>
          {isAllSelected ? "取消全选" : "全选"}
        </SelectAllButton>
      </SelectAllRow>

      <PluginList>
        {onboardingPlugins.map((plugin) => {
          const isSelected = selectedPlugins.includes(plugin.id);
          const Icon = plugin.icon;

          return (
            <PluginCard
              key={plugin.id}
              $selected={isSelected}
              onClick={() => handleToggle(plugin.id)}
            >
              <CheckboxWrapper>
                <Checkbox
                  checked={isSelected}
                  onCheckedChange={() => handleToggle(plugin.id)}
                />
              </CheckboxWrapper>
              <IconWrapper $selected={isSelected}>
                <Icon />
              </IconWrapper>
              <PluginInfo>
                <PluginName>{plugin.name}</PluginName>
                <PluginDescription>{plugin.description}</PluginDescription>
              </PluginInfo>
            </PluginCard>
          );
        })}
      </PluginList>
    </Container>
  );
}
