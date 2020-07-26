动态内存分配： 
与C++中malloc函数作用相类似，可以使操作系统内核以根据程序运行时状态修改内存申请的时机及大小，同时销毁内存块
应用场景： Box Arc 以及std中vec等容器进行内存申请
支持方法：实现支持Trait GlobalAlloc的类，然后将其实例化并使用#[global_allocator] 进行标记


物理内存探测：
默认的 DRAM 物理内存地址范围就是 [0x80000000, 0x88000000)，共128MB的大小
建立一个 PhysicalAddress 的类，然后对其实现一系列的 usize 的加、减和输出等等操作
同时编码出指向内核代码结束位置的指针变量
lazy_static! {
    /// 内核代码结束的地址，即可以用来分配的内存起始地址
    ///
    /// 因为 Rust 语言限制，我们只能将其作为一个运行时求值的 static 变量，而不能作为 const
    pub static ref KERNEL_END_ADDRESS: PhysicalAddress = PhysicalAddress(kernel_end as usize);
}


物理内存管理：
物理页：
实际进行内存分配时并不是按照字节为单位，而是将内存划分为页面，每个页面4K大小。
分配和回收：
页面的追踪
pub struct FrameTracker(PhysicalAddress);
该类实现了两个函数：
     pub fn address(&self) -> PhysicalAddress
     pub fn page_number(&self) -> PhysicalPageNumber  分别返回当前物理地址或页号   
同时
impl Drop for FrameTracker {
    fn drop(&mut self) {
        FRAME_ALLOCATOR.lock().dealloc(self);
    }
}
功能是在析构时自动释放所指向的内存页
ub static ref FRAME_ALLOCATOR: Mutex<FrameAllocator<AllocatorImpl>>
为页面的分配器指针实例化

/// 基于线段树的帧分配 / 回收
pub struct FrameAllocator<T: Allocator> {
    /// 可用区间的起始
    start_ppn: PhysicalPageNumber,
    /// 分配器
    allocator: T,
}
实现了 
    pub fn new(range: impl Into<Range<PhysicalPageNumber>> + Copy) -> Self

    pub fn alloc(&mut self) -> MemoryResult<FrameTracker>

    pub(super) fn dealloc(&mut self, frame: &FrameTracker)

///子模块 frame  主要用于物理内存的分配
/mod.rs
pub use 将后面的跟随的模块中的功能函数引入当前作用域，向外暴露接口
pub use allocator::FRAME_ALLOCATOR;
pub use frame_tracker::FrameTracker;


/++++++++++++++++++++++++++++++++++++++++++++++++++++++++++/
/allocator.rs    //物理内存帧分配器

定义了分配器
pub struct FrameAllocator<T: Allocator> {
    /// 可用区间的起始
    start_ppn: PhysicalPageNumber,
    /// 分配器
    allocator: T,
}
初始化一个静态变量为全局物理内存帧分配器，以后分配内存都是通过FRAME_ALLOCATOR的操作。
lazy_static! {
    /// 帧分配器
    pub static ref FRAME_ALLOCATOR: Mutex<FrameAllocator<AllocatorImpl>> = Mutex::new(FrameAllocator::new(Range::from(
            PhysicalPageNumber::ceil(PhysicalAddress::from(*KERNEL_END_ADDRESS))..PhysicalPageNumber::floor(MEMORY_END_ADDRESS),
        )
    ));
}
//操作内存时需要格外小心，Mutex保证同一个实践最多只有一个线程可以操作内存的分配和释放，避免出错

//其中的 AllocatorImpl  从外部 algorithm导入   //之后介绍

impl<T: Allocator> FrameAllocator<T>{
    //创建一个FrameAllocator<T>对象
    pub fn new(range: impl Into<Range<PhysicalPageNumber>> + Copy)->Self
    /// 分配帧，如果没有剩余则返回 `Err`
    pub fn alloc(&mut self) -> MemoryResult<FrameTracker>

    /// 将被释放的帧添加到空闲列表的尾部
    ///
    /// 这个函数会在 [`FrameTracker`] 被 drop 时自动调用，不应在其他地方调用
    pub(super) fn dealloc(&mut self, frame: &FrameTracker)
     
    //FrameTracker和对应物理帧进行了某种绑定，drop了一个Frametracker之后其所代表的物理帧同时被释放
}


/++++++++++++++++++++++++++++++++++++++++++++++++++++++++++/
/frame_tracker.rs

pub struct FrameTracker(pub(super) PhysicalPageNumber);  //定义  内含物理页编号的封装，其对应了一个物理帧
pub(in super)  代表memory模块可以直接访问 再外层就不行了

impl FrameTracker {
    /// 帧的物理地址
    pub fn address(&self) -> PhysicalAddress {
        self.0.into()
    }
    /// 帧的物理页号
    pub fn page_number(&self) -> PhysicalPageNumber {
        self.0
    }
}

并且FrameTracker支持自动解引用为 &[u8; PAGE_SIZE]   &mut [u8; PAGE_SIZE] 

/// 帧在释放时会放回 [`static@FRAME_ALLOCATOR`] 的空闲链表中
impl Drop for FrameTracker {
    fn drop(&mut self) {
        FRAME_ALLOCATOR.lock().dealloc(self);
    }
}