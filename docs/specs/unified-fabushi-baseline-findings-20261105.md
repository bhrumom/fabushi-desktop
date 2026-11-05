# UFX 基线问题与验收重点

只读源审查；基线 7f6548d42。以下不是运行时验收，不宣称已修复。

1. P1 伪造执行结果：OpenClawAskCopilotPanel 本地 setTimeout 输出成功/调度/连接器就绪；ProductionRenderer 装配仅传 onClose，无真实后端。删除伪执行链，所有能力状态必须来自真实 owner。
2. P1 模型/权限控件缺少真实接线：OpenClawCleanComposer 定义 hardcoded AVAILABLE_MODELS、默认 GPT-6 Astra/Medium/allowlist；ProductionRenderer 的该组件装配不传 model/onChangeModel/thinkingLevel/onChangeThinkingLevel/securityGate/onChangeSecurityGate。视觉选择不能证明实际生效；恢复 canonical editorProviders 与权限 owner，按真实 provider 能力渲染。
3. P1 输入器功能回退风险：同一 ProductionRenderer 的 Human 分支传 acceptedSendGeneration、replyTarget、onClearReplyTarget、onRemoveAttachment、scopeKey 给 ConversationComposer，Agent 分支替换成 OpenClawCleanComposer 未传这些合同。必须逐项检查 accepted clear、回复、附件、scope/草稿、流式取消、IME/语音等；不能只换外观。
4. P2 分享语义错误：OpenClawTopHeader 文案“分享会话”，传入 handler 却选 displayedEntries 最后一条非 card 打开 forwardTarget。应明确“转发消息”并定位所选消息，或走真正会话分享合同；不允许一键意外分享最后私密内容。
5. P2 静态/无作用导航：OpenClawTopHeader 显示默认 workspace 面包屑及缺少实际 back/forward handler 的禁用箭头。按有意义的当前对象动作收口；不保留纯装饰控件。

建议 first-batch oracle：断网时没有默认 ready；实际模型切换收到配置 owner 确认后才反映；回复 target 与附件删除正确；发送成功清空对应会话草稿，失败保留；切换账号后迟到回复/旧草稿不写入新账号；分享动作标注准确范围。

CLI 启动记录：首次 xhigh 被 gemini-4-argon 拒绝，根会话未创建；auto 模式账号不可用。第二次保留原模型，使用其可支持 high（复杂任务 adapter fallback）及 accept-edits，实际 init permission_mode=request-review。根 conversation=6989f3d7-a8d0-4fa9-8c9b-cda7b70c9f88；teamwork coordinator=85e84663-8255-4b2c-8a64-337f2a878b15。expanded_commands 确认 teamwork 为 system command；初始 team workspace 正确。尚不等于 worker 开始修改或测试通过。
