/**
 * 浏览器模式下优先走 mock 的命令集合。
 *
 * 这些命令要么依赖当前 DevBridge 尚未桥接的原生能力，
 * 要么即使缺少真实后端也不应阻塞默认页面渲染。
 */

const mockPriorityCommands = new Set<string>([
  "aster_agent_init",
  "connection_list",
  "terminal_create_session",
  "list_dir",
  "get_plugins_with_ui",
  "get_plugin_status",
  "get_plugins",
  "list_installed_plugins",
  "list_plugin_tasks",
  "get_plugin_queue_stats",
  "subscribe_sysinfo",
  "unsubscribe_sysinfo",
  "session_files_get_or_create",
  "session_files_update_meta",
  "session_files_list_files",
  "session_files_save_file",
  "session_files_read_file",
  "session_files_delete_file",
  "execution_run_get_theme_workbench_state",
  "aster_agent_chat_stream",
  "get_hint_routes",
  "content_workflow_get_by_content",
]);

export function shouldPreferMockInBrowser(cmd: string): boolean {
  return mockPriorityCommands.has(cmd);
}

export { mockPriorityCommands };
