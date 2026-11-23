//! 侧栏顶上的几何：文件树工具栏、折叠工具、视图切换的分段按钮、红绿灯让出的位置。

use super::*;
use crate::app::Focus;
use winc_i18n::t;

/// 文件树工具栏上的按钮。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeTool {
    /// 新建文件。
    NewFile,
    /// 新建目录。
    NewFolder,
    /// 刷新目录。
    Refresh,
    /// 折叠所有目录。
    CollapseAll,
    /// 在文件树中定位当前文件。
    Locate,
}

impl TreeTool {
    /// 全部工具，顺序与工具栏从左到右的顺序一致。
    pub const ALL: [TreeTool; 5] = [
        TreeTool::NewFile,
        TreeTool::NewFolder,
        TreeTool::Refresh,
        TreeTool::CollapseAll,
        TreeTool::Locate,
    ];

    /// 返回悬停提示文字。
    pub fn label(self) -> &'static str {
        match self {
            TreeTool::NewFile => t!("workspace-tree-tool-new-file"),
            TreeTool::NewFolder => t!("workspace-tree-tool-new-folder"),
            TreeTool::Refresh => t!("workspace-tree-tool-refresh"),
            TreeTool::CollapseAll => t!("workspace-tree-tool-collapse-all"),
            TreeTool::Locate => t!("workspace-tree-tool-locate"),
        }
    }
}

/// 项目名（含箭头）所占宽度的下限（逻辑像素）：侧栏最窄时仍要露出名字的开头和折叠箭头，
/// 工具栏不占用这部分宽度。
pub const TREE_NAME_MIN: f32 = 48.0;

/// 返回文件树工具栏的按钮布局：位于项目名所在行的右端，从左到右依次是新建文件、新建目录、刷新、
/// 全部折叠、定位。
///
/// 放不下时从右向左收起：留下的按钮仍然显示，收起的按钮移入「⋯」菜单。返回留下的按钮（从左到右）
/// 和「⋯」按钮的位置，全部放得下时后者为 `None`。`name_right` 是项目名和箭头的右边界，
/// 工具栏不会越过该边界。
pub fn tree_tools(header: Rect, name_right: f32) -> (Vec<(Rect, TreeTool)>, Option<Rect>) {
    let right = header.right() - 8.0;
    // 可容纳的按钮数；有按钮被收起时，其中一格留给「⋯」，刚好全部放下时不显示「⋯」。
    let room = ((right - name_right + BUTTON_GAP) / (BUTTON_SMALL + BUTTON_GAP))
        .floor()
        .max(0.0) as usize;
    let all = TreeTool::ALL.len();
    let button = |i, count| toolbar_button(header, right, i, count);

    if room >= all {
        let buttons = (0..all)
            .map(|i| (button(i, all), TreeTool::ALL[i]))
            .collect();
        return (buttons, None);
    }

    if room == 0 {
        return (Vec::new(), None);
    }

    // 最右侧一格留给「⋯」，其余各格按顺序留给最靠前的按钮。
    let kept = room - 1;
    let buttons = (0..kept)
        .map(|i| (button(i, room), TreeTool::ALL[i]))
        .collect();
    (buttons, Some(button(kept, room)))
}

/// 返回收起动画途中文件树工具栏的绘制位置：`shown` 是各工具（按 [`TreeTool::ALL`] 的顺序）
/// 显示出来的程度，`more` 是「⋯」按钮的，0 表示收起，1 表示完全显示。
///
/// 从右往左排：「⋯」固定在最右一格，每个工具按自身程度占用对应格数。正在收起的工具向右滑到右侧相邻
/// 的按钮上（最终叠进「⋯」），左边的按钮跟着右移补上空位；程度都是 0 或 1 时与
/// [`TreeState::tools`] 的结果相同。返回显示程度大于 0 的工具（从左到右）及其程度，以及「⋯」按钮。
pub fn folding_tools(
    header: Rect,
    shown: &[f32; TreeTool::ALL.len()],
    more: f32,
) -> (Vec<(Rect, TreeTool, f32)>, Option<Rect>) {
    let right = header.right() - 8.0;
    let step = BUTTON_SMALL + BUTTON_GAP;
    let y = header.y + (header.h - BUTTON_SMALL) / 2.0;

    let mut cursor = right - more * step;
    let mut tools = Vec::new();
    for (i, &tool) in TreeTool::ALL.iter().enumerate().rev() {
        if shown[i] <= 0.0 {
            continue;
        }
        cursor -= shown[i] * step;
        let rect = Rect::new(cursor + BUTTON_GAP, y, BUTTON_SMALL, BUTTON_SMALL);
        tools.push((rect, tool, shown[i]));
    }
    tools.reverse();

    let more = (more > 0.0).then(|| toolbar_button(header, right, 0, 1));
    (tools, more)
}

/// 返回右对齐于 `right` 的一排 `count` 个工具栏按钮中第 `i` 个（从 0 起）的位置，在 `bar`
/// 内垂直居中。
pub(in crate::layout) fn toolbar_button(bar: Rect, right: f32, i: usize, count: usize) -> Rect {
    let step = BUTTON_SMALL + BUTTON_GAP;
    let x = right - (count - i) as f32 * step + BUTTON_GAP;
    let y = bar.y + (bar.h - BUTTON_SMALL) / 2.0;
    Rect::new(x, y, BUTTON_SMALL, BUTTON_SMALL)
}

/// 返回工具栏收起后移入「⋯」菜单的按钮；`kept` 是工具栏留下的按钮数。
pub fn folded_tools(kept: usize) -> &'static [TreeTool] {
    &TreeTool::ALL[kept.min(TreeTool::ALL.len())..]
}

/// 侧栏显示的内容：文件树、全局搜索，或 Git 的提交视图。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SidebarView {
    /// 文件树。
    #[default]
    Files,
    /// 全局搜索面板。
    Search,
    /// Git 的提交视图：改动列表、提交信息。
    ///
    /// 只在 Git 模块开着、目录在仓库里时有。
    Commit,
}

/// 一个侧栏视图的全部登记信息：显示名、切换它的命令、点开之后焦点落在哪里、分段里的图标。
///
/// 新增视图时在这里添一项，并补上 `SidebarView` 变体对应的绘制：分段控件的顺序、悬停提示、
/// 快捷键、命令面板里的条目、点击分段后的动作、焦点落点都从这一张表来，不再分散到各处 match。
pub struct SidebarPage {
    /// 视图本身，也是分段控件里的顺序。
    pub view: SidebarView,
    /// 切换它的命令：承担快捷键、命令面板条目和悬停提示里的按键。
    pub action: Action,
    /// 点开后焦点落在哪里。
    pub focus: Focus,
    /// 分段里的图标；`scale` 为主轴尺寸，`clip` 是允许绘制的范围。
    pub icon: fn(&mut Frame<'_>, (f32, f32), f32, Color, Rect),
}

impl SidebarPage {
    /// 返回悬停提示文字（含快捷键）。
    pub fn label(&self) -> String {
        with_key(self.view.title(), Some(self.action))
    }
}

/// 总是有的两个视图，顺序即视图切换里的顺序。
pub const BASE_PAGES: [&SidebarPage; 2] = [
    &SidebarPage {
        view: SidebarView::Files,
        action: Action::ShowFiles,
        focus: Focus::Tree,
        icon: icons::files,
    },
    &SidebarPage {
        view: SidebarView::Search,
        action: Action::FindInFiles,
        focus: Focus::GlobalQuery,
        icon: icons::search,
    },
];

/// 目录在仓库里才有的提交视图。它没有固定的焦点，点开时由 `App::show_commit` 决定。
pub const COMMIT_PAGE: SidebarPage = SidebarPage {
    view: SidebarView::Commit,
    action: Action::ShowCommit,
    focus: Focus::CommitList,
    icon: icons::git,
};

impl SidebarView {
    /// 返回视图的名字，用于分段按钮的悬停提示。
    pub fn title(self) -> &'static str {
        match self {
            SidebarView::Files => t!("workspace-sidebar-files"),
            SidebarView::Search => t!("workspace-sidebar-search"),
            SidebarView::Commit => t!("workspace-sidebar-commit"),
        }
    }

    /// 返回这个视图的登记信息。
    pub fn page(self) -> &'static SidebarPage {
        match self {
            SidebarView::Files => BASE_PAGES[0],
            SidebarView::Search => BASE_PAGES[1],
            SidebarView::Commit => &COMMIT_PAGE,
        }
    }

    /// 返回悬停提示文字（含快捷键）。
    pub fn label(self) -> String {
        self.page().label()
    }
}

/// 红绿灯右端距窗口左边缘的距离（逻辑像素），以 `f32` 的位存放；见 [`TRAFFIC_LIGHTS_END`]。
static LIGHTS_END: AtomicU32 = AtomicU32::new(TRAFFIC_LIGHTS_END.to_bits());

/// 返回红绿灯右端距窗口左边缘的距离（逻辑像素）；没有红绿灯时为 0。
pub fn traffic_lights_end() -> f32 {
    f32::from_bits(LIGHTS_END.load(Ordering::Relaxed))
}

/// 记录红绿灯右端的实际位置（窗口创建、摆放红绿灯后调用），返回位置是否有变化。
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub fn set_traffic_lights_end(end: f32) -> bool {
    LIGHTS_END.swap(end.to_bits(), Ordering::Relaxed) != end.to_bits()
}

/// 返回红绿灯右侧第一个按钮的左边缘：侧栏收起时是侧栏开关，展开时是侧栏顶部的「文件」切换。
///
/// 两个按钮里的图标宽度不同（16 和 14），因此按图标可见的左边缘对齐，而不是按按钮对齐：侧栏
/// 收起和展开时，红绿灯到第一个图标的距离都是 [`LIGHTS_GAP`]。没有红绿灯时按钮距卡片左边缘 8。
pub(in crate::layout) fn first_button_x(icon_width: f32) -> f32 {
    let end = traffic_lights_end();
    if end > 0.0 {
        end + LIGHTS_GAP - (BUTTON - icon_width) / 2.0
    } else {
        ISLAND_MARGIN + 8.0
    }
}

/// 视图切换（分段控件）里一段的宽度（逻辑像素）：图标左右宽松，段与段之间隔
/// [`SEGMENT_SPACING`]，外面的底槽每边再宽 2。
pub const SEGMENT_W: f32 = 28.0;
/// 视图切换里一段的高度（逻辑像素）：22 高的横条，图标上下各留 4。
pub const SEGMENT_H: f32 = 22.0;
/// 分段控件里相邻两段之间的缝：选中一段凸起时与旁边一段分开，不贴在一起。
pub const SEGMENT_SPACING: f32 = 2.0;
/// 视图切换和侧栏开关之间至少隔多远（侧栏最窄时）。
pub const SEGMENT_GAP: f32 = 6.0;

/// 返回顶栏里视图切换（「文件」「搜索」以及模块提供的视图）每一段的位置，顺序同 `views`：
/// 分段控件靠侧栏 `sidebar` 的右端（离右边 8 像素，与第二行右端的按钮对齐）；侧栏窄到放不下
/// 全部段与它和开关之间的 [`SEGMENT_GAP`] 时，多出的段不再硬挤，而是等宽压缩（见
/// [`segment_w`]），保证与开关之间、与侧栏右端都留出原有的距离。
///
/// 每一段带一个显示比例（0–1），平时是 1：模块的视图出现、消失途中，对应的一段和前面的缝
/// 按比例变窄，底槽跟着伸缩，整个控件仍靠右。
pub fn view_tabs(
    toggle: Rect,
    sidebar: Rect,
    views: &[(SidebarView, f32)],
) -> Vec<(Rect, SidebarView)> {
    // 每段前面的缝；第一段前面没有。
    let gap = |i: usize| if i == 0 { 0.0 } else { SEGMENT_SPACING };
    let natural = views
        .iter()
        .enumerate()
        .map(|(i, v)| (SEGMENT_W + gap(i)) * v.1)
        .sum::<f32>()
        + 4.0;
    let room = sidebar.right() - 8.0 - toggle.right() - SEGMENT_GAP;
    let w = segment_w(views.len(), room);
    let gaps = SEGMENT_SPACING * (views.len().saturating_sub(1)) as f32;
    // 压缩时整条按实际段宽重排，宽度之和不超过 `room`；放得下时按原样从右端排开。
    let track_w = if w < SEGMENT_W {
        w * views.len() as f32 + gaps + 4.0
    } else {
        natural
    };
    let left = sidebar.right() - 8.0 - track_w + 2.0;
    let y = toggle.y + (toggle.h - SEGMENT_H) / 2.0;
    let mut x = left;
    views
        .iter()
        .enumerate()
        .map(|(i, &(view, share))| {
            x += gap(i) * share;
            let rect = Rect::new(x, y, w * share, SEGMENT_H);
            x += w * share;
            (rect, view)
        })
        .collect()
}

/// 返回视图切换里每一段的宽度：放得下时是 [`SEGMENT_W`]，`room` 不够时等宽压缩。
///
/// 侧栏窄到连图标都放不下时照样按比例缩，按钮里的图标由绘制方按段宽裁剪，不会压到旁边一段上。
pub fn segment_w(count: usize, room: f32) -> f32 {
    let room = room.max(0.0);
    if count <= 1 {
        return SEGMENT_W.min(room);
    }
    let natural = SEGMENT_W * count as f32 + SEGMENT_SPACING * (count - 1) as f32;
    if natural <= room {
        return SEGMENT_W;
    }
    ((room - SEGMENT_SPACING * (count - 1) as f32) / count as f32).max(0.0)
}
