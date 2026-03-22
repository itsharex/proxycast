export interface BuiltinInputCommand {
  key: "image";
  label: string;
  mentionLabel: string;
  commandPrefix: string;
  description: string;
  aliases: string[];
}

export const INPUTBAR_BUILTIN_COMMANDS: BuiltinInputCommand[] = [
  {
    key: "image",
    label: "配图",
    mentionLabel: "配图",
    commandPrefix: "@配图",
    description: "进入图片生成、编辑与变体工作台",
    aliases: ["image", "img", "图片", "生图"],
  },
];

export function filterBuiltinCommands(
  query: string,
): BuiltinInputCommand[] {
  const normalizedQuery = query.trim().toLowerCase();
  if (!normalizedQuery) {
    return INPUTBAR_BUILTIN_COMMANDS;
  }

  return INPUTBAR_BUILTIN_COMMANDS.filter((command) => {
    const haystacks = [
      command.label,
      command.mentionLabel,
      command.commandPrefix,
      command.description,
      ...command.aliases,
    ];
    return haystacks.some((value) =>
      value.toLowerCase().includes(normalizedQuery),
    );
  });
}

