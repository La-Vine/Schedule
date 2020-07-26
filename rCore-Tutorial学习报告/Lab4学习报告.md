进程调度

基本概念： 
进程是得到了操作系统的资源支持，并且使用计算机资源的程序；程序的代码、数据段被加载到内存中，程序所需的虚拟内存空间被真正构建出来。
同时操作系统还给进程分配了程序所要求的各种其他资源，如我们上面几个章节中提到过的页表、文件的资源。
为了能够进行函数调用，我们还需要运行栈（Stack）
这样的一个借助 CPU 和栈的执行流，我们称之为线程 (Thread) 

线程的表示

/// 线程的信息
pub struct Thread {
    /// 线程 ID
    pub id: ThreadID,
    /// 线程的栈
    pub stack: Range<VirtualAddress>,
    /// 线程执行上下文
    ///
    /// 当且仅当线程被暂停执行时，`context` 为 `Some`
    pub context: Mutex<Option<Context>>,
    /// 所属的进程
    pub process: Arc<RwLock<Process>>,
}

进程的表示
/// 进程的信息
pub struct Process {
    /// 是否属于用户态
    pub is_user: bool,
    /// 进程中的线程公用页表 / 内存映射
    pub memory_set: MemorySet,  //进程中的线程会共享同一个页表，即互相可以访问空间。
}

线程的创建
1.建立页表映射，需要包括以下映射空间：
    线程所执行的一段指令
    线程执行栈
    操作系统的部分内存空间
2.设置起始执行的地址
3.初始化各种寄存器，比如 sp
4.可选：设置一些执行参数（例如 argc 和 argv等 ）

内核栈
1.预留一段空间作为内核栈
2.运行线程时，在 sscratch 寄存器中保存内核栈指针
3.如果线程遇到中断，则从将 Context 压入 sscratch 指向的栈中（Context 的地址为 sscratch - 5.size_of::<Context>()），同时用新的栈地址来替换 sp（此时 sp 也会被复制到 a0 作为 handle_interrupt 的参数）
4.从中断中返回时（__restore 时），a0 应指向被压在内核栈中的 Context。此时出栈 Context 并且将栈顶保存到 sscratch 中

/// 内核栈
#[repr(align(16))]
#[repr(C)]
pub struct KernelStack([u8; KERNEL_STACK_SIZE]);

/// 公用的内核栈
pub static mut KERNEL_STACK: KernelStack = KernelStack([0; STACK_SIZE]);

process模块报告
/++++++++++++++++++++++++++++++++++++++++++++++++++++++++++/
/config.rs
//! 定义一些进程相关的常量
/// 每个线程的运行栈大小 512 KB
pub const STACK_SIZE: usize = 0x8_0000;
/// 共用的内核栈大小 512 KB
pub const KERNEL_STACK_SIZE: usize = 0x8_0000;

/++++++++++++++++++++++++++++++++++++++++++++++++++++++++++/
/mod.rs
像模块外声明了几个可调用名称

/++++++++++++++++++++++++++++++++++++++++++++++++++++++++++/
/thread.rs
线程相关

/// 线程的信息
pub struct Thread {
    /// 线程 ID
    pub id: ThreadID,     //type = isize
    /// 线程的栈
    pub stack: Range<VirtualAddress>,
    /// 所属的进程
    pub process: Arc<RwLock<Process>>,
    /// 用 `Mutex` 包装一些可变的变量
    pub inner: Mutex<ThreadInner>,
}

/// 线程中需要可变的部分
pub struct ThreadInner {
    /// 线程执行上下文
    ///
    /// 当且仅当线程被暂停执行时，`context` 为 `Some`
    pub context: Option<Context>,
    /// 是否进入休眠
    pub sleeping: bool,
    /// 打开的文件
    pub descriptors: Vec<Arc<dyn INode>>,
}

static mut THREAD_COUNTER: ThreadID = 0;

impl Thread
    pub fn prepare(&self) -> *mut Context//将当前线程准备运行
        1.激活对应进程的memory_set
        2.取出context
        3.将context放入栈中准备弹出进行线程切换 1） 用户线程  放在内核栈顶
                                            2）内核线程  放在sp下

        
    /// 发生时钟中断后暂停线程，保存状态
    pub fn park(&self, context: Context)

    /// 创建一个线程
    pub fn new(
        process: Arc<RwLock<Process>>,
        entry_point: usize,
        arguments: Option<&[usize]>,
    ) -> MemoryResult<Arc<Thread>>

    1.让当前进程分配线程栈
    2.构建context
    3.打包为Arc<Thread>
    4.返回OK(thread)

    //返回ThreadInner
    pub fn inner(&self) -> spin::MutexGuard<ThreadInner> {
        self.inner.lock()
    }

为了能够让进程调度器正确工作
还impl 了 PartialEq,Eq和Hash
/++++++++++++++++++++++++++++++++++++++++++++++++++++++++++/
process.rs
/// 进程的信息
pub struct Process {
    /// 是否属于用户态
    pub is_user: bool,
    /// 进程中的线程公用页表 / 内存映射
    pub memory_set: MemorySet,
}

impl Process
    /// 创建一个内核进程
    pub fn new_kernel() -> MemoryResult<Arc<RwLock<Self>>>
        Self{is_user:false,memory_set:MemorySet::new_kernel()?}

    /// 创建进程，从文件中读取代码
    pub fn from_elf(file: &ElfFile, is_user: bool) -> MemoryResult<Arc<RwLock<Self>>>
        Self {
            is_user, //is_user同名
            memory_set: MemorySet::from_elf(file, is_user)? //通过from_elf来分配memory_set
        }
    /// 分配一定数量的连续虚拟空间
    ///
    /// 从 `memory_set` 中找到一段给定长度的未占用虚拟地址空间，分配物理页面并建立映射。返回对应的页面区间。
    ///
    /// `flags` 只需包括 rwx 权限，user 位会根据进程而定。
    pub fn alloc_page_range(
        &mut self,
        size: usize,
        flags: Flags,
    ) -> MemoryResult<Range<VirtualAddress>>

/++++++++++++++++++++++++++++++++++++++++++++++++++++++++++/
/kernel_stack.rs
内核栈
/// 内核栈
#[repr(align(16))]
#[repr(C)]
pub struct KernelStack([u8; KERNEL_STACK_SIZE]);
/// 公用的内核栈
pub static KERNEL_STACK: KernelStack = KernelStack([0; KERNEL_STACK_SIZE]);

/// 在栈顶加入 Context 并且返回新的栈顶指针
    pub fn push_context(&self, context: Context) -> *mut Context


/++++++++++++++++++++++++++++++++++++++++++++++++++++++++++/
processor.rs
//调度工作
#[derive(Default)]
pub struct Processor {
    /// 当前正在执行的线程
    current_thread: Option<Arc<Thread>>,
    /// 线程调度器，记录活跃线程
    scheduler: SchedulerImpl<Arc<Thread>>,
    /// 保存休眠线程
    sleeping_threads: HashSet<Arc<Thread>>,
}