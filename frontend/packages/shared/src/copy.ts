export const homeHighlights = [
  {
    title: "经文听诵",
    description: "读经、听诵、离线素材与进度记录，日常修行更顺手。",
  },
  {
    title: "全球法布施",
    description: "把善意发送到世界各地，用 3D 地球看见传播路径。",
  },
  {
    title: "禅修与法流",
    description: "禅室、冥想、短视频法流与修行记录，轻量进入完整体验。",
  },
] as const;

export const homeUseCases = [
  {
    audience: "日常修行",
    title: "打开就能读经、听诵、记录。",
    description: "适合想把修行放进每天固定节奏的人。",
  },
  {
    audience: "佛法传播",
    title: "用可视化方式看见法布施。",
    description: "适合想分享经文、参与全球发送或关注功德回向的人。",
  },
  {
    audience: "共修同行",
    title: "看见记录，也保留边界。",
    description: "适合想参与共修、保持个人记录和控制公开范围的人。",
  },
  {
    audience: "内测体验",
    title: "快速找到对应下载入口。",
    description: "适合想试用 Android Beta 或 iOS TestFlight 的用户。",
  },
] as const;

export const primaryNavigation = [
  { label: "首页", href: "/" },
  { label: "下载", href: "/download" },
  { label: "申请测试", href: "/apply" },
  { label: "常见问题", href: "/faq" },
  { label: "联系", href: "/contact" },
] as const;

export const faqItems = [
  {
    question: "大乘 是什么？",
    answer:
      "大乘 是一款佛教修行应用，提供经文听诵、全球法布施、禅修冥想、法流视频和修行记录。",
  },
  {
    question: "现在可以下载吗？",
    answer:
      "Android Beta 和 iOS TestFlight 会在下载页显示当前状态。入口开放时可以直接点击；未开放时可以先申请测试。",
  },
  {
    question: "Android 和 iOS 入口有什么区别？",
    answer:
      "Android Beta 通常更快同步版本；iOS 通过 TestFlight 加入内测；正式版会在人工确认后公开。",
  },
  {
    question: "适合哪些人先试用？",
    answer:
      "适合需要日常读经听诵、禅修记录、佛法内容传播，或愿意参与早期体验并反馈问题的人。",
  },
  {
    question: "测试申请需要提供什么？",
    answer:
      "请写明平台、设备型号、常用邮箱和最想体验的功能。合作沟通可直接说明方向和联系方式。",
  },
  {
    question: "下载或安装遇到问题怎么办？",
    answer:
      "可以通过 support@ombhrum.com 联系支持。请附上平台、系统版本、错误截图或发生步骤。",
  },
] as const;

export const contactChannels = [
  {
    label: "支持邮箱",
    value: "support@ombhrum.com",
    href: "mailto:support@ombhrum.com",
    note: "下载问题、测试申请、账号支持和合作沟通。",
  },
  {
    label: "官网域名",
    value: "fabushi.ombhrum.com",
    href: "https://fabushi.ombhrum.com",
    note: "长期公开入口，适合转发下载页和常见问题。",
  },
  {
    label: "公开项目",
    value: "fabushi.ombhrum.com",
    href: "https://fabushi.ombhrum.com",
    note: "查看项目进展、功能更新和最新发布动态。",
  },
] as const;

export const betaApplicationTracks = [
  {
    name: "iOS TestFlight",
    summary: "适合想体验完整主应用，并愿意接受内测节奏的用户。",
    checklist: [
      "附上你的常用邮箱",
      "说明你更关注内容传播、修行记录还是社交发现",
      "写明你是否愿意反馈 bug 和体验问题",
    ],
    ctaLabel: "申请 iOS 内测",
    ctaHref:
      "mailto:support@ombhrum.com?subject=Fabushi%20iOS%20Beta%20Application&body=%E4%BD%A0%E5%A5%BD%EF%BC%8C%E6%88%91%E6%83%B3%E7%94%B3%E8%AF%B7%20Fabushi%20iOS%20%E5%86%85%E6%B5%8B%E3%80%82%0A%0A%E5%B8%B8%E7%94%A8%E9%82%AE%E7%AE%B1%EF%BC%9A%0A%E5%85%B3%E6%B3%A8%E7%82%B9%EF%BC%9A%0A%E6%98%AF%E5%90%A6%E6%84%BF%E6%84%8F%E5%8F%8D%E9%A6%88%E9%97%AE%E9%A2%98%EF%BC%9A%0A",
  },
  {
    name: "Android Beta",
    summary: "适合想尽快体验新版本、全球法布施和法流内容的人。",
    checklist: [
      "附上你的常用邮箱",
      "说明你的 Android 机型或系统版本",
      "写明你最想优先体验哪个模块",
    ],
    ctaLabel: "申请 Android 内测",
    ctaHref:
      "mailto:support@ombhrum.com?subject=Fabushi%20Android%20Beta%20Application&body=%E4%BD%A0%E5%A5%BD%EF%BC%8C%E6%88%91%E6%83%B3%E7%94%B3%E8%AF%B7%20Fabushi%20Android%20%E5%86%85%E6%B5%8B%E3%80%82%0A%0A%E5%B8%B8%E7%94%A8%E9%82%AE%E7%AE%B1%EF%BC%9A%0AAndroid%20%E6%9C%BA%E5%9E%8B%2F%E7%B3%BB%E7%BB%9F%E7%89%88%E6%9C%AC%EF%BC%9A%0A%E6%9C%80%E6%83%B3%E4%BD%93%E9%AA%8C%E7%9A%84%E6%A8%A1%E5%9D%97%EF%BC%9A%0A",
  },
  {
    name: "合作与渠道",
    summary: "适合讨论传播合作、内容共建、渠道联动或活动承接。",
    checklist: [
      "附上你的姓名或组织名称",
      "说明合作方向或渠道资源",
      "留下可回联的邮箱或微信说明",
    ],
    ctaLabel: "发起合作沟通",
    ctaHref:
      "mailto:support@ombhrum.com?subject=Fabushi%20Partnership%20Inquiry&body=%E4%BD%A0%E5%A5%BD%EF%BC%8C%E6%88%91%E6%83%B3%E5%92%8C%20Fabushi%20%E8%AE%A8%E8%AE%BA%E5%90%88%E4%BD%9C%E3%80%82%0A%0A%E5%A7%93%E5%90%8D%E6%88%96%E7%BB%84%E7%BB%87%E5%90%8D%E7%A7%B0%EF%BC%9A%0A%E5%90%88%E4%BD%9C%E6%96%B9%E5%90%91%EF%BC%9A%0A%E5%9B%9E%E8%81%94%E6%96%B9%E5%BC%8F%EF%BC%9A%0A",
  },
] as const;
