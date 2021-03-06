#[macro_use]
extern crate nom;
extern crate num_bigint;
extern crate num_traits;

use nom::*;
use self::num_bigint::BigInt;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::str;
use std::str::FromStr;
use std::fs::File;
use std::io::Read;
use datalayout::*;
use helper::*;
use types::*;
use num_traits::cast::FromPrimitive;
use std::cmp::min;

pub mod datalayout;
pub mod types;
mod helper;
#[cfg(test)]
mod tests;

pub type Alignment = u64;
pub type AttributeGroup = u64;

#[derive(Debug,PartialEq,Eq,PartialOrd,Ord,Hash,Clone,Copy)]
pub enum Linkage {
    Private,
    Internal,
    AvailableExternally,
    LinkOnce,
    Weak,
    Common,
    Appending,
    ExternWeak,
    LinkOnceODR,
    WeakODR,
    External
}

#[derive(Debug,PartialEq,Eq,PartialOrd,Ord,Hash,Clone,Copy)]
pub enum Visibility {
    Default,
    Hidden,
    Protected
}

#[derive(Debug,PartialEq,Eq,PartialOrd,Ord,Hash,Clone,Copy)]
pub enum DLLStorageClass {
    Default,
    DLLImport,
    DLLExport
}

#[derive(Debug,PartialEq,Eq,PartialOrd,Ord,Hash,Clone,Copy)]
pub enum ThreadLocal {
    ThreadLocal,
    LocalDynamic,
    InitialExec,
    LocalExec
}

#[derive(Debug,PartialEq,Eq,PartialOrd,Ord,Hash,Clone,Copy)]
pub enum UnnamedAddr {
    UnnamedAddr,
    LocalUnnamedAddr
}

#[derive(Debug,PartialEq,Eq,PartialOrd,Ord,Hash,Clone,Copy)]
pub enum GlobalType {
    Global,
    Constant
}

#[derive(Debug,PartialEq,Eq,PartialOrd,Ord,Hash,Clone)]
pub struct GlobalVariable {
    pub linkage: Option<Linkage>,
    pub visibility: Visibility,
    pub dll_storage_class: DLLStorageClass,
    pub thread_local: Option<ThreadLocal>,
    pub unnamed_addr: Option<UnnamedAddr>,
    pub addr_space: Option<AddressSpace>,
    pub externally_initialized: bool,
    pub global_type: GlobalType,
    pub types: Type,
    pub initialization: Option<Constant>,
    pub section: Option<String>,
    pub alignment: Option<Alignment>,
}

#[derive(Debug,PartialEq,Eq,PartialOrd,Ord,Hash,Clone)]
pub enum Constant {
    Global(String),
    Int(BigInt),
    Array(Vec<Constant>),
    GEP(Box<GEP<Constant>>),
    NullPtr
}

#[derive(Debug,PartialEq,Eq,PartialOrd,Ord,Hash,Clone)]
pub struct GEP<T> {
    pub ptr: Typed<T>,
    pub inbounds: bool,
    pub indices: Vec<(Typed<T>,bool)>
}

#[derive(Debug,PartialEq,Eq,PartialOrd,Ord,Hash,Clone)]
pub struct Attribute {
    pub name: String,
    pub quoted: bool,
    pub value: Option<String>
}

#[derive(Debug,PartialEq,Eq,Clone)]
pub struct Module {
    pub id: Option<String>,
    pub datalayout: DataLayout,
    pub triple: Option<String>,
    pub functions: HashMap<String,Function>,
    pub types: HashMap<String,Type>,
    pub globals: HashMap<String,GlobalVariable>,
    pub attr_groups: HashMap<u64,Vec<Attribute>>,
    pub named_md: HashMap<String,Metadata>,
    pub md: HashMap<u64,Metadata>
}

#[derive(Debug,PartialEq,Eq,Clone)]
pub struct Function {
    pub name: String,
    pub linkage: Option<Linkage>,
    pub visibility: Visibility,
    pub dll_storage_class: DLLStorageClass,
    pub cconv: CallingConv,
    pub return_type: Option<(ParAttrs,Type)>,
    pub arguments: Vec<(Option<String>,Type)>,
    pub var_args: bool,
    pub attribute_groups: Vec<AttributeGroup>,
    pub body: Option<Vec<BasicBlock>>
}

impl Function {
    fn is_defined(&self) -> bool {
        self.body.is_some()
    }
}

#[derive(Debug,PartialEq,Eq,Clone)]
pub struct BasicBlock {
    pub name: String,
    pub instrs: Vec<Instruction>
}

#[derive(Debug,PartialEq,Eq,Clone)]
pub struct Instruction {
    pub content: InstructionC,
    pub metadata: HashMap<String,u64>
}

#[derive(Debug,PartialEq,Eq,PartialOrd,Ord,Hash,Clone)]
pub enum CmpOp {
    Eq,Ne,
    UGt,UGe,ULt,ULe,
    SGt,SGe,SLt,SLe
}

#[derive(Debug,PartialEq,Eq,PartialOrd,Ord,Hash,Clone)]
pub enum CallingConv {
    C,
    Fast,
    Cold,
    WebKitJS,
    AnyReg,
    PreserveMost,
    PreserveAll,
    CxxFastTLS,
    Swift,
    Numbered(u64)
}

#[derive(Debug,PartialEq,Eq,PartialOrd,Ord,Hash,Clone)]
pub struct ParAttrs {
    pub zeroext: bool,
    pub signext: bool,
    pub inreg: bool,
    pub byval: bool,
    pub inalloca: bool,
    pub sret: bool,
    pub align: Option<Alignment>,
    pub noalias: bool,
    pub nocapture: bool,
    pub nest: bool,
    pub returned: bool,
    pub nonnull: bool,
    pub dereferenceable: Option<u64>,
    pub dereferenceable_or_null: Option<u64>,
    pub swiftself: bool,
    pub swifterror: bool
}

impl ParAttrs {
    pub fn new() -> ParAttrs {
        ParAttrs { zeroext: false,
                   signext: false,
                   inreg: false,
                   byval: false,
                   inalloca: false,
                   sret: false,
                   align: None,
                   noalias: false,
                   nocapture: false,
                   nest: false,
                   returned: false,
                   nonnull: false,
                   dereferenceable: None,
                   dereferenceable_or_null: None,
                   swiftself: false,
                   swifterror: false }
    }
}

#[derive(Debug,PartialEq,Eq,PartialOrd,Ord,Hash,Clone)]
pub enum BinOp {
    Add(bool,bool),
    Sub(bool,bool),
    Mul(bool,bool),
    And,Or,XOr,
    AShr,LShr,Shl,
    SDiv(bool)
}

#[derive(Debug,PartialEq,Eq,PartialOrd,Ord,Hash,Clone)]
pub enum Terminator {
    Br(String),
    BrC(Value,String,String),
    Ret(Option<Typed<Value>>),
    Switch(Type,Value,String,Vec<(Constant,String)>),
    Unreachable
}

#[derive(Debug,PartialEq,Eq,PartialOrd,Ord,Hash,Clone)]
pub enum InstructionC {
    Alloca(String,Type,Option<Typed<Value>>,Option<Alignment>),
    Call(Option<String>,CallingConv,Option<(Type,ParAttrs)>,Value,Vec<Typed<Value>>,Vec<AttributeGroup>),
    ICmp(String,CmpOp,Type,Value,Value),
    Unary(String,Typed<Value>,UnaryInst),
    GEP(String,GEP<Value>),
    Store(bool,Typed<Value>,Typed<Value>,Option<Alignment>),
    Select(String,Value,Type,Value,Value),
    Phi(String,Type,Vec<(Value,String)>),
    Bin(String,BinOp,Type,Value,Value),
    Term(Terminator)
}

#[derive(Debug,PartialEq,Eq,PartialOrd,Ord,Hash,Clone)]
pub enum UnaryInst {
    Cast(Type,CastInst),
    Load(bool,Option<Alignment>),
}

#[derive(Debug,PartialEq,Eq,PartialOrd,Ord,Hash,Clone,Copy)]
pub enum CastInst {
    Trunc,ZExt,SExt,Bitcast,IntToPtr,PtrToInt
}

#[derive(Debug,PartialEq,Eq,PartialOrd,Ord,Hash,Clone)]
pub struct Typed<T> {
    pub tp: Type,
    pub val: T
}

impl<T> Typed<T> {
    fn new(tp: Type,val: T) -> Typed<T> {
        Typed { tp: tp, val: val }
    }
}

#[derive(Debug,PartialEq,Eq,PartialOrd,Ord,Hash,Clone)]
pub enum Value {
    Constant(Constant),
    Local(String),
    Argument(usize),
    Metadata(Metadata)
}

#[derive(Debug,PartialEq,Eq,PartialOrd,Ord,Hash,Clone)]
pub enum Metadata {
    Null,
    Ref(u64),
    Value(Box<Typed<Value>>),
    Struct(Vec<Metadata>),
    Bytes(Vec<u8>),
    Location(u64,u64,Box<Metadata>)
}

const NO_ARGS: [(Option<String>,Type); 0] = [];

named!(module_id<&str>,
       map_res!(delimited!( tag!("; ModuleID = \'"),
                            is_not!("\'"),
                            char!('\'') ),
                str::from_utf8));

named!(triple<&str>,
       map_res!( ws!(do_parse!( tag!("target") >>
                                tag!("triple") >>
                                char!('=') >>
                                char!('"') >>
                                trp: is_not!("\"") >>
                                char!('"') >>
                                (trp) )),
                     str::from_utf8));

named!(type_def<(&str,Type)>,
       ws!(do_parse!( name: local_name >>
                      char!('=') >>
                      tag!("type") >>
                      tp: types >>
                      (name,tp))));

named!(global_def<(&str,GlobalVariable)>,
       do_parse!( name: global_name >>
                  llvm_space >>
                  char!('=') >>
                  llvm_space >>
                  glob: global_variable >>
                  (name,glob)));

named!(function_definition<(&str,Function)>,
       do_parse!(is_defined: alt!(map!(tag!("define"),|_| true) |
                                  map!(tag!("declare"),|_| false)) >>
                 llvm_space >>
                 lnk: opt!(terminated!(linkage,llvm_space)) >>
                 vis: alt!(terminated!(visibility,llvm_space) |
                           value!(Visibility::Default)) >>
                 stcls: alt!(terminated!(dll_storage_class,llvm_space) |
                             value!(DLLStorageClass::Default)) >>
                 cc: alt!(terminated!(calling_conv,llvm_space) |
                          value!(CallingConv::C)) >>
                 ret: alt!(map!(tag!("void"),
                                |_| None) |
                           map!(pair!(par_attrs,types),Some)) >>
                 llvm_space >>
                 name: global_name >>
                 llvm_space >>
                 char!('(') >>
                 llvm_space >>
                 args: separated_list!(delimited!(llvm_space,char!(','),llvm_space),
                                       do_parse!(tp: types >>
                                                 n: opt!(preceded!(llvm_space,map!(local_name,|n| n.to_string()))) >>
                                                 (n,tp))) >>
                 va: map!( opt!(do_parse!( cond!(!args.is_empty(),
                                                 terminated!(char!(','),llvm_space)) >>
                                           tag!("...") >>
                                           ())),
                           |x| x.is_some()) >>
                 llvm_space >>
                 char!(')') >>
                 llvm_space >>
                 attrs: many0!(delimited!(char!('#'),parse_u64,llvm_space)) >>
                 llvm_space >>
                 blks: cond!(is_defined,
                             do_parse!(char!('{') >>
                                       llvm_nl >>
                                       blks: many0!(terminated!(call!(basic_block,&args[..]),llvm_nl)) >>
                                       char!('}') >>
                                       (blks))) >>
                 (name,Function { name: name.to_string(),
                                  linkage: lnk,
                                  visibility: vis,
                                  dll_storage_class: stcls,
                                  cconv: cc,
                                  return_type: ret,
                                  arguments: args,
                                  var_args: va,
                                  attribute_groups: attrs,
                                  body: blks })));

named!(comment,
       preceded!(char!(';'),
                 is_not!("\n")));

named_args!(basic_block<'a>(args: &'a [(Option<String>,Type)])<BasicBlock>,
       do_parse!(name: map_res!(is_not!("} \t\n:"),
                                str::from_utf8) >>
                 char!(':') >>
                 instrs: many0!(preceded!(llvm_nl,alt!(call!(instruction,args) |
                                                       preceded!(not!(alt!(char!('}') | preceded!(is_not!("} \t\n:"),char!(':')))),
                                                                 map!(map_res!(is_not!("\n"),str::from_utf8),
                                                                      |s| { panic!("Cannot parse instruction: {}",s) }))))) >>
                 (BasicBlock { name: name.to_string(),
                               instrs: instrs })));

named_args!(instruction<'a>(args: &'a [(Option<String>,Type)])<Instruction>,
       do_parse!(cont: call!(instruction_c,args) >>
                 meta: fold_many0!(do_parse!(llvm_space >>
                                             char!(',') >>
                                             llvm_space >>
                                             char!('!') >>
                                             name: map_res!(is_not!(" \t\r\n"),
                                                            str::from_utf8) >>
                                             llvm_space >>
                                             char!('!') >>
                                             id: parse_u64 >>
                                             (name.to_string(),id)),
                                   HashMap::new(),
                                   |mut mp: HashMap<String,u64>,(name,id)| {
                                       mp.insert(name,id);
                                       mp
                                   }) >>
                 (Instruction { content: cont,
                                metadata: meta })));

named_args!(call<'a>(ctx_args: &'a [(Option<String>,Type)])
            <(CallingConv,Option<(Type,ParAttrs)>,Value,Vec<Typed<Value>>,Vec<AttributeGroup>)>,
            do_parse!(tag!("call") >>
                      llvm_space >>
                      cc: alt!(terminated!(calling_conv,llvm_space) |
                               value!(CallingConv::C)) >>
                      pattrs: par_attrs >>
                      rtp: alt!(map!(tag!("void"),
                                     |_| None) |
                                map!(types,|t| Some((t,pattrs)))) >>
                      llvm_space >>
                      fun: call!(value,ctx_args) >>
                      llvm_space >>
                      char!('(') >>
                      llvm_space >>
                      args: separated_list!(terminated!(char!(','),
                                                        llvm_space),
                                            terminated!(call!(typed_value,ctx_args),
                                                        llvm_space)) >>
                      char!(')') >>
                      attrs: many0!(do_parse!(llvm_space >> char!('#') >> r: parse_u64 >> (r))) >>
                      (cc,rtp,fun,args,attrs)));

named!(cmp_op<CmpOp>,
       alt!( map!(tag!("eq"),|_| CmpOp::Eq) |
             map!(tag!("ne"),|_| CmpOp::Ne) |
             map!(tag!("ugt"),|_| CmpOp::UGt) |
             map!(tag!("uge"),|_| CmpOp::UGe) |
             map!(tag!("ult"),|_| CmpOp::ULt) |
             map!(tag!("ule"),|_| CmpOp::ULe) |
             map!(tag!("sgt"),|_| CmpOp::SGt) |
             map!(tag!("sge"),|_| CmpOp::SGe) |
             map!(tag!("slt"),|_| CmpOp::SLt) |
             map!(tag!("sle"),|_| CmpOp::SLe)));

named_args!(instruction_c<'a>(args: &'a [(Option<String>,Type)])<InstructionC>,
       alt_complete!(map!(call!(call,args),
                 |(cc,rtp,fun,call_args,attrs)|
                 InstructionC::Call(None,cc,rtp,fun,call_args,attrs)) |
            do_parse!(tag!("br") >>
                      llvm_space >>
                      res: alt!( do_parse!(tag!("label") >>
                                           llvm_space >>
                                           name: local_name >>
                                           (InstructionC::Term(Terminator::Br(name.to_string())))) |
                                 do_parse!(tag!("i1") >>
                                           llvm_space >>
                                           c: call!(value,args) >>
                                           llvm_space >>
                                           char!(',') >>
                                           llvm_space >>
                                           tag!("label") >>
                                           llvm_space >>
                                           l1: local_name >>
                                           llvm_space >>
                                           char!(',') >>
                                           llvm_space >>
                                           tag!("label") >>
                                           llvm_space >>
                                           l2: local_name >>
                                           (InstructionC::Term(Terminator::BrC(c,l1.to_string(),
                                                                               l2.to_string()))))) >>
                      (res)) |
            map!(tag!("unreachable"),
                 |_| InstructionC::Term(Terminator::Unreachable)) |
            do_parse!(tag!("store") >>
                      llvm_space >>
                      vol: map!(opt!(terminated!(tag!("volatile"),
                                                 llvm_space)),
                                |x| x.is_some()) >>
                      obj: call!(typed_value,args) >>
                      llvm_space >>
                      char!(',') >>
                      llvm_space >>
                      ptr: call!(typed_value,args) >>
                      llvm_space >>
                      align: alignment >>
                      (InstructionC::Store(vol,obj,ptr,align))) |
            do_parse!(tag!("ret") >>
                      llvm_space >>
                      rval: alt_complete!( map!(call!(typed_value,args),Some) |
                                           map!(tag!("void"),|_| None)) >>
                      (InstructionC::Term(Terminator::Ret(rval)))) |
            do_parse!(tag!("switch") >>
                      llvm_space >>
                      tp: types >>
                      llvm_space >>
                      val: call!(value,args) >>
                      llvm_space >>
                      char!(',') >>
                      llvm_space >>
                      tag!("label") >>
                      llvm_space >>
                      def: local_name >>
                      llvm_space >>
                      char!('[') >>
                      jmps: many0!(do_parse!(llvm_ws >>
                                             types >>
                                             llvm_space >>
                                             v: constant >>
                                             llvm_space >>
                                             char!(',') >>
                                             llvm_space >>
                                             tag!("label") >>
                                             llvm_space >>
                                             lbl: local_name >>
                                             (v,lbl.to_string()))) >>
                      llvm_ws >>
                      char!(']') >>
                      (InstructionC::Term(Terminator::Switch(tp,val,def.to_string(),jmps)))) |
            do_parse!(name: local_name >>
                      llvm_space >>
                      char!('=') >>
                      llvm_space >>
                      cont: alt!(map!(call!(call,args),
                                      |c| InstructionC::Call(Some(name.to_string()),c.0,c.1,c.2,c.3,c.4)) |
                                 do_parse!(tag!("icmp") >>
                                           llvm_space >>
                                           op: cmp_op >>
                                           llvm_space >>
                                           tp: types >>
                                           llvm_space >>
                                           v1: call!(value,args) >>
                                           llvm_space >>
                                           char!(',') >>
                                           llvm_space >>
                                           v2: call!(value,args) >>
                                           (InstructionC::ICmp(name.to_string(),
                                                               op,tp,v1,v2))) |
                                 do_parse!(tag!("load") >>
                                           llvm_space >>
                                           vol: map!(opt!(terminated!(tag!("volatile"),
                                                                      llvm_space)),
                                                     |x| x.is_some()) >>
                                           ptr: call!(typed_value,args) >>
                                           llvm_space >>
                                           align: alignment >>
                                           (InstructionC::Unary(name.to_string(),
                                                                ptr,
                                                                UnaryInst::Load(vol,align)))) |
                                 do_parse!(op: alt!(map!(tag!("trunc"),|_| CastInst::Trunc) |
                                                    map!(tag!("zext"),|_| CastInst::ZExt) |
                                                    map!(tag!("sext"),|_| CastInst::SExt) |
                                                    map!(tag!("bitcast"),|_| CastInst::Bitcast) |
                                                    map!(tag!("inttoptr"),|_| CastInst::IntToPtr) |
                                                    map!(tag!("ptrtoint"),|_| CastInst::PtrToInt)) >>
                                           llvm_space >>
                                           val: call!(typed_value,args) >>
                                           llvm_space >>
                                           tag!("to") >>
                                           llvm_space >>
                                           trg: types >>
                                           (InstructionC::Unary(name.to_string(),
                                                                val,
                                                                UnaryInst::Cast(trg,op)))) |
                                 map!(call!(gep,|inp| value(inp,args),false),
                                      |g| InstructionC::GEP(name.to_string(),g)) |
                                 do_parse!(tag!("select") >>
                                           llvm_space >>
                                           tag!("i1") >>
                                           llvm_space >>
                                           cond: call!(value,args) >>
                                           llvm_space >>
                                           char!(',') >>
                                           llvm_space >>
                                           tp1: types >>
                                           llvm_space >>
                                           v1: call!(value,args) >>
                                           llvm_space >>
                                           char!(',') >>
                                           llvm_space >>
                                           types >>
                                           llvm_space >>
                                           v2: call!(value,args) >>
                                           (InstructionC::Select(name.to_string(),
                                                                 cond,tp1,v1,v2))) |
                                 do_parse!(tag!("phi") >>
                                           llvm_space >>
                                           tp: types >>
                                           llvm_space >>
                                           trgs: separated_list!(do_parse!(llvm_space >> char!(',') >> llvm_space >> ()),
                                                                 do_parse!(char!('[') >>
                                                                           llvm_space >>
                                                                           v: call!(value,args) >>
                                                                           llvm_space >>
                                                                           char!(',') >>
                                                                           llvm_space >>
                                                                           blk: local_name >>
                                                                           llvm_space >>
                                                                           char!(']') >>
                                                                           ((v,blk.to_string())))) >>
                                           (InstructionC::Phi(name.to_string(),
                                                              tp,trgs))) |
                                 do_parse!(op: alt!(do_parse!(tag!("add") >>
                                                              nuw: map!(opt!(preceded!(llvm_space,tag!("nuw"))),
                                                                        |x| x.is_some()) >>
                                                              nsw: map!(opt!(preceded!(llvm_space,tag!("nsw"))),
                                                                        |x| x.is_some()) >>
                                                              (BinOp::Add(nuw,nsw))) |
                                                    do_parse!(tag!("sub") >>
                                                              nuw: map!(opt!(preceded!(llvm_space,tag!("nuw"))),
                                                                        |x| x.is_some()) >>
                                                              nsw: map!(opt!(preceded!(llvm_space,tag!("nsw"))),
                                                                        |x| x.is_some()) >>
                                                              (BinOp::Sub(nuw,nsw))) |
                                                    do_parse!(tag!("mul") >>
                                                              nuw: map!(opt!(preceded!(llvm_space,tag!("nuw"))),
                                                                        |x| x.is_some()) >>
                                                              nsw: map!(opt!(preceded!(llvm_space,tag!("nsw"))),
                                                                        |x| x.is_some()) >>
                                                              (BinOp::Mul(nuw,nsw))) |
                                                    map!(tag!("and"),|_| BinOp::And) |
                                                    map!(tag!("or"),|_| BinOp::Or) |
                                                    map!(tag!("xor"),|_| BinOp::XOr) |
                                                    map!(tag!("ashr"),|_| BinOp::AShr) |
                                                    map!(tag!("lshr"),|_| BinOp::LShr) |
                                                    map!(tag!("shl"),|_| BinOp::Shl) |
                                                    do_parse!(tag!("sdiv") >>
                                                              exact: map!(opt!(preceded!(llvm_space,tag!("exact"))),
                                                                          |x| x.is_some()) >>
                                                              (BinOp::SDiv(exact)))) >>
                                           llvm_space >>
                                           tp: types >>
                                           llvm_space >>
                                           v1: call!(value,args) >>
                                           llvm_space >>
                                           char!(',') >>
                                           llvm_space >>
                                           v2: call!(value,args) >>
                                           (InstructionC::Bin(name.to_string(),
                                                              op,tp,v1,v2))) |
                                 do_parse!(tag!("alloca") >>
                                           llvm_space >>
                                           tp: types >>
                                           llvm_space >>
                                           num: opt!(preceded!(terminated!(char!(','),llvm_space),
                                                               call!(typed_value,args))) >>
                                           align: alignment >>
                                           (InstructionC::Alloca(name.to_string(),tp,num,align)))
                      ) >>
                      (cont))
       ));

named_args!(typed_value<'a>(args: &'a [(Option<String>,Type)])<Typed<Value>>,
       alt_complete!(do_parse!(tag!("metadata")>>
                               llvm_space >>
                               r: call!(metadata,args) >>
                               (Typed::new(Type::Metadata,Value::Metadata(r)))) |
                     do_parse!(tp: types >>
                               llvm_space >>
                               v: call!(value,args) >>
                               (Typed::new(tp,v)))));

/*named!(typed_constant<Typed<Constant>>,
       ws!(do_parse!(tp: types >>
                     c: constant >>
                     (Typed::new(tp,c)))));*/

named_args!(argument<'a>(args: &'a [(Option<String>,Type)])<usize>,
            map_opt!(local_name,|name:&str| { let rname = name.to_string();
                                              args.iter().position(|&(ref arg_name,_)| {
                                                  match arg_name {
                                                      &Some(ref oname) => *oname==rname,
                                                      &None => false
                                                  }
                                              }) }));

named_args!(value<'a>(args: &'a [(Option<String>,Type)])<Value>,
            alt_complete!(map!(call!(argument,args),
                               |num| Value::Argument(num)) |
                          map!(local_name,
                               |name| Value::Local(name.to_string())) |
                          map!(constant,
                               Value::Constant)
            ));

named_args!(metadata<'a>(args: &'a [(Option<String>,Type)])<Metadata>,
       alt_complete!(map!(tag!("null"),|_| Metadata::Null) |
                     preceded!(char!('!'),
                               alt!(do_parse!(char!('{') >>
                                              llvm_space >>
                                              els: separated_list!(delimited!(llvm_space,char!(','),llvm_space),
                                                                   call!(metadata,args)) >>
                                              llvm_space >>
                                              char!('}') >>
                                              (Metadata::Struct(els))) |
                                    map!(parse_u64,Metadata::Ref) |
                                    map!(delimited!(char!('"'),
                                                    many0!(alt!(map_res!(map_res!(preceded!(char!('\\'),take!(2)),
                                                                                  str::from_utf8),
                                                                         FromStr::from_str) |
                                                                map_opt!(be_u8,|c| if c==b'"' { None } else { Some(c) }))),
                                                    char!('"')),
                                         Metadata::Bytes) |
                                    do_parse!(tag!("MDLocation") >>
                                              llvm_space >>
                                              char!('(') >>
                                              llvm_space >>
                                              tag!("line:") >>
                                              llvm_space >>
                                              l: parse_u64 >>
                                              llvm_space >>
                                              char!(',') >>
                                              llvm_space >>
                                              tag!("column:") >>
                                              llvm_space >>
                                              c: parse_u64 >>
                                              llvm_space >>
                                              char!(',') >>
                                              llvm_space >>
                                              tag!("scope:") >>
                                              llvm_space >>
                                              sc: call!(metadata,args) >>
                                              llvm_space >>
                                              char!(')') >>
                                              (Metadata::Location(l,c,Box::new(sc)))))) |
                     map!(call!(typed_value,args),
                          |v| Metadata::Value(Box::new(v))))); 

named!(attribute<Attribute>,
       do_parse!(name: alt!(map!(map_res!(alpha,str::from_utf8),
                                 |s| (s.to_string(),false)) |
                            map!(delimited!(char!('\"'),
                                            map_res!(is_not!("\""),
                                                     str::from_utf8),
                                            char!('\"')),
                                 |s| (s.to_string(),true))) >>
                 val: opt!(do_parse!(char!('=') >>
                                     s: delimited!(char!('\"'),
                                                   map_res!(is_not!("\""),
                                                            str::from_utf8),
                                                   char!('\"')) >>
                                     (s.to_string()))) >>
                 (Attribute { name: name.0,
                              quoted: name.1,
                              value: val })));

named!(attribute_group<(u64,Vec<Attribute>)>,
       do_parse!(tag!("attributes") >>
                 llvm_space >>
                 char!('#') >>
                 n: parse_u64 >>
                 llvm_space >>
                 char!('=') >>
                 llvm_space >>
                 char!('{') >>
                 llvm_space >>
                 attrs: many0!(terminated!(attribute,llvm_space)) >>
                 char!('}') >>
                 (n,attrs)));

named_args!(named_metadata<'a>(args: &'a [(Option<String>,Type)])<(String,Metadata)>,
       do_parse!(char!('!') >>
                 name: map_res!(is_not!(" =!,\n"),
                                str::from_utf8) >>
                 llvm_space >>
                 char!('=') >>
                 llvm_space >>
                 def: call!(metadata,args) >>
                 (name.to_string(),def)));

named_args!(num_metadata<'a>(args: &'a [(Option<String>,Type)])<(u64,Metadata)>,
       do_parse!(char!('!') >>
                 name: parse_u64 >>
                 llvm_space >>
                 char!('=') >>
                 llvm_space >>
                 def: call!(metadata,args) >>
                 (name,def)));

named_args!(module_element<'a>(m: &'a mut Module)<()>,
            alt!( map!(module_id,
                       |id| {
                           m.id = Some(id.to_string());
                       }) |
                  map!(datalayout,
                       |dl| {
                           m.datalayout = dl;
                       }) |
                  map!(triple,
                       |tr| {
                           m.triple = Some(tr.to_string());
                       }) |
                  map!(type_def,
                       |(name,tp)| {
                           m.types.insert(name.to_string(),tp);
                       }) |
                  map!(global_def,
                       |(name,def)| {
                           m.globals.insert(name.to_string(),def);
                       }) |
                  map!(function_definition,
                       |(name,fun)| {
                           match m.functions.entry(name.to_string()) {
                               Entry::Occupied(mut e) => if !e.get().is_defined() {
                                   e.insert(fun);
                               },
                               Entry::Vacant(e) => { e.insert(fun); }
                           }
                       }) |
                  map!(attribute_group,
                       |(n,attrs)| {
                           m.attr_groups.insert(n,attrs);
                       }) |
                  map!(call!(num_metadata,&NO_ARGS),
                       |(n,md)| {
                           m.md.insert(n,md);
                       }) |
                  map!(call!(named_metadata,&NO_ARGS),
                       |(n,md)| {
                           m.named_md.insert(n,md);
                       }) |
                  map!(comment,
                       |_| { })));

fn gep<T,F>(input: &[u8],parse: F,paren: bool) -> IResult<&[u8],GEP<T>>
    where F : Fn(&[u8]) -> IResult<&[u8],T> {
    do_parse!(input,
              tag!("getelementptr") >>
              llvm_space >>
              inb: map!(opt!(terminated!(tag!("inbounds"),llvm_space)),
                        |x| x.is_some()) >>
              cond!(paren,terminated!(char!('('),
                                      llvm_space)) >>
              tp: types >>
              llvm_space >>
              ptr: call!(parse) >>
              llvm_space >>
              idx: many0!(do_parse!(char!(',') >>
                                    llvm_space >>
                                    ir: map!(opt!(terminated!(tag!("inrange"),llvm_space)),
                                             |x| x.is_some()) >>
                                    tp: types >>
                                    v: call!(parse) >>
                                    llvm_space >>
                                    (Typed::new(tp,v),ir))) >>
              cond!(paren,char!(')')) >>
              (GEP { ptr: Typed::new(tp,ptr), inbounds: inb, indices: idx }))
}

named!(constant<Constant>,
       alt_complete!( map!(tag!("null"),
                  |_| Constant::NullPtr) |
             map!(tag!("false"),
                  |_| Constant::Int(BigInt::from(0))) |
             map!(tag!("true"),
                  |_| Constant::Int(BigInt::from(1))) |
             map!(global_name,
                  |name| Constant::Global(name.to_string())) |
             do_parse!(char!('c') >>
                       char!('\"') >>
                       res: fold_many0!(map!(constant_char,
                                             |c| Constant::Int(c)),
                                        Vec::new(),
                                        |mut vec: Vec<Constant>,el| {
                                            vec.push(el);
                                            vec }) >>
                       char!('\"') >>
                       (Constant::Array(res))) |
             map!(map_opt!(digit,
                           |s| { BigInt::parse_bytes(s,10) }),
                  Constant::Int) |
             map!(map_opt!(preceded!(char!('-'),digit),
                           |s| { BigInt::parse_bytes(s,10) }),
                  |i| Constant::Int(-i)) |
             map!(call!(gep,constant,true),
                  |g| Constant::GEP(Box::new(g)))
       ));

named!(constant_char<BigInt>,
       alt!(map_opt!(preceded!(char!('\\'),
                               take!(2)),
                     |s| { BigInt::parse_bytes(s,16) }) |
            map_opt!(take!(1),
                     |x: &[u8]| if x[0]==b'"' {
                         None
                     } else {
                         BigInt::from_u8(x[0])
                     })));

named!(linkage<Linkage>,
       alt!( map!(tag!("private"),|_| Linkage::Private) |
             map!(tag!("internal"),|_| Linkage::Internal) |
             map!(tag!("available_externally"),|_| Linkage::AvailableExternally) |
             map!(tag!("linkonce"),|_| Linkage::LinkOnce) |
             map!(tag!("weak"),|_| Linkage::Weak) |
             map!(tag!("common"),|_| Linkage::Common) |
             map!(tag!("appending"),|_| Linkage::Appending) |
             map!(tag!("extern_weak"),|_| Linkage::ExternWeak) |
             map!(tag!("linkonce_odr"),|_| Linkage::LinkOnceODR) |
             map!(tag!("weak_odr"),|_| Linkage::WeakODR) |
             map!(tag!("external"),|_| Linkage::External) ));

named!(visibility<Visibility>,
       alt!( map!(tag!("default"),|_| Visibility::Default) |
             map!(tag!("hidden"),|_| Visibility::Hidden) |
             map!(tag!("protected"),|_| Visibility::Protected)));

named!(dll_storage_class<DLLStorageClass>,
       alt!( map!(tag!("default"),|_| DLLStorageClass::Default) |
             map!(tag!("dllimport"),|_| DLLStorageClass::DLLImport) |
             map!(tag!("dllexport"),|_| DLLStorageClass::DLLExport)));

named!(thread_local<ThreadLocal>,
       alt!( map!(ws!(do_parse!(tag!("thread") >>
                                tag!("local") >>
                                ())),
                  |_| ThreadLocal::ThreadLocal) |
             map!(tag!("localdynamic"),
                  |_| ThreadLocal::LocalDynamic) |
             map!(tag!("initialexec"),
                  |_| ThreadLocal::InitialExec) |
             map!(tag!("localexec"),
                  |_| ThreadLocal::LocalExec)));

named!(unnamed_addr<UnnamedAddr>,
       alt!( map!(tag!("unnamed_addr"),
                  |_| UnnamedAddr::UnnamedAddr) |
             map!(tag!("local_unnamed_addr"),
                  |_| UnnamedAddr::LocalUnnamedAddr)));

named!(externally_initialized<()>,
       map!(tag!("externally_initialized"),
            |_| ()));

named!(global_type<GlobalType>,
       alt!(map!(tag!("global"),
                 |_| GlobalType::Global) |
            map!(tag!("constant"),
                 |_| GlobalType::Constant)));

named!(alignment<Option<Alignment>>,
       opt!(complete!(do_parse!(char!(',') >>
                                llvm_space >>
                                tag!("align") >>
                                llvm_space >>
                                n: parse_u64 >>
                                llvm_space >>
                                (n)))));
            
named!(global_variable<GlobalVariable>,
       do_parse!(l: opt!(terminated!(linkage,llvm_space)) >>
                 v: alt!(terminated!(visibility,llvm_space) |
                         value!(Visibility::Default)) >>
                 dll: alt!(terminated!(dll_storage_class,llvm_space) |
                           value!(DLLStorageClass::Default)) >>
                 loc: opt!(terminated!(thread_local,llvm_space)) >>
                 ua: opt!(terminated!(unnamed_addr,llvm_space)) >>
                 addrsp: opt!(terminated!(address_space,llvm_space)) >>
                 ext: map!(opt!(terminated!(externally_initialized,llvm_space)),
                           |v| { match v {
                               Some(_) => true,
                               None => false } }) >>
                 gtp: global_type >>
                 llvm_space >>
                 tp: types >>
                 llvm_space >>
                 init: opt!(terminated!(
                     alt!(do_parse!(tag!("zeroinitializer") >>
                                    (Constant::zero_init(&tp))) |
                          constant),
                     llvm_space)) >>
                 sec: opt!(do_parse!(char!(',') >>
                                     llvm_space >>
                                     tag!("section") >>
                                     llvm_space >>
                                     char!('"') >>
                                     name: map_res!(map_res!(is_not!("\""),
                                                             str::from_utf8),
                                                    String::from_str) >>
                                     char!('"') >>
                                     llvm_space >>
                                     (name))) >>
                 align: alignment >>
                 (GlobalVariable { linkage: l,
                                   visibility: v,
                                   dll_storage_class: dll,
                                   thread_local: loc,
                                   unnamed_addr: ua,
                                   addr_space: addrsp,
                                   externally_initialized: ext,
                                   global_type: gtp,
                                   types: tp,
                                   initialization: init,
                                   section: sec,
                                   alignment: align })));

named!(calling_conv<CallingConv>,
       alt!( map!(tag!("ccc"),|_| CallingConv::C) |
             map!(tag!("fastcc"),|_| CallingConv::Fast) |
             map!(tag!("coldcc"),|_| CallingConv::Cold) |
             map!(tag!("webkit_jscc"),|_| CallingConv::WebKitJS) |
             map!(tag!("anyregcc"),|_| CallingConv::AnyReg) |
             map!(tag!("preserve_mostcc"),|_| CallingConv::PreserveMost) |
             map!(tag!("preserve_allcc"),|_| CallingConv::PreserveAll) |
             map!(tag!("cxx_fast_tlscc"),|_| CallingConv::CxxFastTLS) |
             map!(tag!("swiftcc"),|_| CallingConv::Swift) |
             do_parse!(tag!("cc") >>
                       llvm_space >>
                       n: parse_u64 >>
                       (CallingConv::Numbered(n)))));

named_args!(par_attr<'a>(attrs: &'a mut ParAttrs)<()>,
            alt!( map!(tag!("zeroext"),
                       |_| { attrs.zeroext = true;
                             () }) |
                  map!(tag!("signext"),
                       |_| { attrs.signext = true;
                             () }) |
                  map!(tag!("inreg"),
                       |_| { attrs.inreg = true;
                             () }) |
                  map!(tag!("byval"),
                       |_| { attrs.byval = true;
                             () }) |
                  map!(tag!("noalias"),
                       |_| { attrs.noalias = true;
                             () })
            ));

fn par_attrs(inp: &[u8]) -> IResult<&[u8],ParAttrs> {
    let mut attrs = ParAttrs::new();
    let mut input = inp;
    while input.len() > 0 {
        match par_attr(input,&mut attrs) {
            IResult::Done(ninp,_) => {
                match llvm_space(ninp) {
                    IResult::Done(ninp2,_) => { input = ninp2; },
                    IResult::Error(err) => return IResult::Error(err),
                    IResult::Incomplete(need) => return IResult::Incomplete(need)
                }
            },
            IResult::Error(_) => return IResult::Done(input,attrs),
            IResult::Incomplete(need) => return IResult::Incomplete(need)
        }
    }
    IResult::Incomplete(Needed::Unknown)
}

struct ModuleBuilder {
    m: Module,
    st: ConsumerState<(),(),Move>
}

impl ModuleBuilder {
    fn new() -> ModuleBuilder {
        ModuleBuilder { m: Module { id: None,
                                    datalayout: DataLayout::new(),
                                    triple: None,
                                    functions: HashMap::new(),
                                    types: HashMap::new(),
                                    globals: HashMap::new(),
                                    attr_groups: HashMap::new(),
                                    named_md: HashMap::new(),
                                    md: HashMap::new() },
                        st: ConsumerState::Continue(Move::Consume(0)) }
    }
}

impl<'a> Consumer<&'a [u8],(),(),Move> for ModuleBuilder {
    fn handle(&mut self,input: Input<&[u8]>) -> &ConsumerState<(),(),Move> {
        match input {
            Input::Eof(None) => {
                println!("EOF");
                self.st = ConsumerState::Done(Move::Consume(0),());
                &self.st
            },
            Input::Empty => {
                println!("Empty");
                self.st = ConsumerState::Continue(Move::Consume(0));
                &self.st
            },
            Input::Element(sl) | Input::Eof(Some(sl)) => {
                {
                    let strs = str::from_utf8(sl).expect("cannot parse utf8");
                    println!("Handle {}",strs);
                }
                match module_element(sl,&mut self.m) {
                    IResult::Done(rest,()) => {
                        {
                            let rest_strs = str::from_utf8(rest).expect("cannot parse utf8");
                            println!("Done: {}",rest_strs);
                            println!("Consumed: {}",sl.offset(rest));
                        }
                        let mut ninp = rest;
                        while ninp.len() > 0 && (ninp[0]==b' ' || ninp[0]==b'\t' || ninp[0]==b'\n') {
                            ninp = &ninp[1..];
                        }
                        self.st = ConsumerState::Continue(Move::Consume(sl.offset(ninp)));
                        &self.st
                    },
                    IResult::Incomplete(n) => {
                        println!("Incomplete");
                        self.st = ConsumerState::Continue(Move::Await(n));
                        &self.st
                    },
                    IResult::Error(_) => {
                        println!("Error");
                        self.st = ConsumerState::Error(());
                        &self.st
                    }
                }
            }
        }
    }
    fn state(&self) -> &ConsumerState<(),(),Move> {
        &self.st
    }
}

pub fn parse_module(file: &str) -> Option<Module> {
    let mut buf = Vec::new();
    let mut f = match File::open(file) {
        Ok(r) => r,
        Err(_) => return None
    };
    match f.read_to_end(&mut buf) {
        Ok(_) => {},
        Err(_) => return None
    }
    match module(&buf[..]) {
        IResult::Done(ninp,m) => if ninp.len()==0 { Some(m) } else { None },
        _ => None
    }
    /*let mut fp = FileProducer::new(file,1024).expect("Cannot open file");
    let mut builder = ModuleBuilder::new();
    loop {
        match fp.apply(&mut builder) {
            &ConsumerState::Error(_) => return None,
            &ConsumerState::Done(_,_) => return Some(builder.m),
            &ConsumerState::Continue(_) => {}
        }
    }*/
}

pub fn module(input: &[u8]) -> IResult<&[u8],Module> {
    let mut inp = input;
    let mut m = Module { id: None,
                         datalayout: DataLayout::new(),
                         triple: None,
                         functions: HashMap::new(),
                         types: HashMap::new(),
                         globals: HashMap::new(),
                         attr_groups: HashMap::new(),
                         named_md: HashMap::new(),
                         md: HashMap::new() };
    while !inp.is_empty() {
        match module_element(inp,&mut m) {
            IResult::Done(ninp,()) => {
                inp = ninp;
                while inp.len() > 0 && (inp[0]==b' ' || inp[0]==b'\t' || inp[0]==b'\n') {
                    inp = &inp[1..];
                }
            },
            IResult::Error(_) => panic!("Not parsed: {:?}",
                                        str::from_utf8(&inp[..min(inp.len(),120)])),
            //return IResult::Error(err),
            IResult::Incomplete(_) => panic!("Not parsed: {:?}",
                                             str::from_utf8(&inp[..min(inp.len(),120)]))
            //return IResult::Incomplete(need)
        }
    }
    IResult::Done(&b""[..],m)
}

impl Constant {
    pub fn zero_init(tp: &Type) -> Self {
        match tp {
            &Type::Int(bw) => Constant::Int(BigInt::from(0)),
            &Type::Pointer(..) => Constant::NullPtr,
            &Type::Array(sz,ref stp) => {
                let mut rvec = Vec::new();
                rvec.resize(sz as usize,
                            Constant::zero_init(stp));
                Constant::Array(rvec)
            },
            _ => panic!("zero_init not implemented for {:?}",tp)
        }
    }
}
