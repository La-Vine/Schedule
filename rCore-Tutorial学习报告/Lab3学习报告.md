从虚拟内存到物理内存：
虚拟地址 --> 映射机构 --> 实际内存地址
核为所有应用进程使用的内存资源做了抽象，形成了一个从虚拟的地址空间到实际地址空间的自动转换功能，这样子可以极大的简化用户态程序的设计，减少了上层程序的大量麻烦。
虚拟地址：用户程序访问的地址
操作系统负责其中的映射
实现映射的方法就是页表

选择 RISC-V 本身硬件支持的 Sv39 模式作为页表的实现

修改内核:
将原来linker script 和之前在物理内存管理上的一些参数修改一下。
/* 数据存放起始地址 */
BASE_ADDRESS = 0xffffffff80200000; /* 修改为虚拟地址 */
/// 内核使用线性映射的偏移量
pub const KERNEL_MAP_OFFSET: usize = 0xffff_ffff_0000_0000;

实现页表:
实现Sv39页表，将一个分配好的物理页拿来把数据填充作为页表，页表中的每一项是一个8字节的页表项

构建了通过虚拟页号获得三级 VPN 的函数

impl VirtualPageNumber {
    /// 得到一、二、三级页号
    pub fn levels(self) -> [usize; 3] {
        [
            self.0.get_bits(18..27),
            self.0.get_bits(9..18),
            self.0.get_bits(0..9),
        ]
    }
}

实现内核重映射：
Segment:内存段
使用 enum 和 struct 来封装内存段映射的类型和内存段本身
/// 映射的类型
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MapType {
    /// 线性映射，操作系统使用
    Linear,
    /// 按帧分配映射
    Framed,
}

/// 一个映射片段（对应旧 tutorial 的 `MemoryArea`）
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Segment {
    /// 映射类型
    pub map_type: MapType,
    /// 所映射的虚拟地址
    pub range: Range<VirtualAddress>,
    /// 权限标志
    pub flags: Flags,
}
mapping:映射 处理虚拟页，物理页的映射

MemorySet：进程存放「它看到的虚拟内存空间分成的内存段」和「这些段中包含的虚拟页到物理页的映射」
/// 一个进程所有关于内存空间管理的信息
pub struct MemorySet {
    /// 维护页表和映射关系
    pub mapping: Mapping,
    /// 每个字段
    pub segments: Vec<Segment>,
}

///子模块 mapping 主要用于虚拟内存的映射
/mod.rs

//! 内存映射
//!
//! 每个线程保存一个 [`Mapping`]，其中记录了所有的字段 [`Segment`]。
//! 同时，也要追踪为页表或字段分配的所有物理页，目的是 drop 掉之后可以安全释放所有资源。


/++++++++++++++++++++++++++++++++++++++++++++++++++++++++++/
/segment.rs
描述了内存段，映射类型（线性/非线性）  虚拟地址范围   权限
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Segment {
    /// 映射类型
    pub map_type: MapType,
    /// 所映射的虚拟地址
    pub range: Range<VirtualAddress>,
    /// 权限标志
    pub flags: Flags,
}

impl
    pub fn iter_mapped(&self) -> Option<impl Iterator<Item = PhysicalPageNumber>>
    对于内核段，可以将某segment中连续的虚拟页导出为物理页帧
    对于其他段，返回None
    pub fn page_range(&self) -> Range<VirtualPageNumber>
    获得对一段物理地址，转换为页区间

此处与range.rs的语法细节需注意

/++++++++++++++++++++++++++++++++++++++++++++++++++++++++++/
/page_table_entry.rs
页表项
#[derive(Copy, Clone, Default)]
pub struct PageTableEntry(usize);   //长度为64
实现了常见的几种功能，详细不再列出


/++++++++++++++++++++++++++++++++++++++++++++++++++++++++++/
/page_table.rs
页表结构，一般不再单独在栈上或堆上分配，而是直接从物理页面分配器中取出一页与Tracker做绑定
#[repr(C)]
pub struct PageTable {
    pub entries: [PageTableEntry; PAGE_SIZE / 8],
}
pub struct PageTableTracker(pub FrameTracker);   //不可Copy

impl PageTableTracker {
    /// 将一个分配的帧清零，形成空的页表
    pub fn new(frame: FrameTracker) -> Self {
        let mut page_table = Self(frame);
        page_table.zero_init();   //为什么可以这样子使用？ 因为PageTableTracker在下面实现了自动derefmut，返回了一个PageTable的可变引用
        page_table
    }
    /// 获取物理页号
    pub fn page_number(&self) -> PhysicalPageNumber {
        self.0.page_number()
    }
}

/++++++++++++++++++++++++++++++++++++++++++++++++++++++++++/
/mapping.rs
#[derive(Default)]
/// 某个进程的内存映射关系
pub struct Mapping {
    /// 保存所有使用到的页表
    page_tables: Vec<PageTableTracker>,
    /// 根页表的物理页号
    root_ppn: PhysicalPageNumber,
}


/++++++++++++++++++++++++++++++++++++++++++++++++++++++++++/
/memory_set.rs

/// 一个进程所有关于内存空间管理的信息
pub struct MemorySet {
    /// 维护页表和映射关系
    pub mapping: Mapping,
    /// 每个字段
    pub segments: Vec<Segment>,
    /// 所有分配的物理页面映射信息
    pub allocated_pairs: Vec<(VirtualPageNumber, FrameTracker)>,
}

pub fn new_kernel() -> MemoryResult<MemorySet> 对内核重映射，新建进程应该首先使用
pub fn from_elf(file: &ElfFile, is_user: bool) -> MemoryResult<MemorySet>  从elf文件新建内存映射
