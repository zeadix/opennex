/** 一个可创建的终端 shell(后端 list_shells 返回,跨平台检测)。 */
export interface ShellInfo {
  /** 展示名:"zsh"、"PowerShell 7"、"Git Bash"、"WSL · Ubuntu" */
  name: string;
  /** 可执行文件完整路径 */
  program: string;
  /** 启动参数(Unix 登录 "-l";WSL 发行版 ["-d", name];cmd/pwsh 为空) */
  args: string[];
}
