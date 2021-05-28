use crate::value::Value;
use crate::state::{State, StateStatus};
use crate::sims::fs::FileMode;

const MAX_LEN: u64 = 8192;

// TODO everything that interacts with errno in any way
// I forget how errno works, i know its weird 

// now using sim fs
pub fn puts(state: &mut State, args: Vec<Value>) -> Value {
    let addr = &args[0];
    let length = strlen(state, vec!(addr.clone()));
    let mut data = state.memory.read_sym_len(addr, &length);
    data.push(Value::Concrete('\n' as u64)); // add newline
    //println!("{}", value);
    state.filesystem.write(1, data);
    length
}

// TODO you know, all this
pub fn printf(state: &mut State, args: Vec<Value>) -> Value {
    puts(state, args)
}

pub fn memmove(state: &mut State, args: Vec<Value>) -> Value {
    state.memory.memmove(&args[0], &args[1], &args[2]);
    args[0].clone()
}

pub fn memcpy(state: &mut State, args: Vec<Value>) -> Value {
    // TODO make actual memcpy that does overlaps right
    // how often do memcpys actually do that? next to never probably
    state.memory.memmove(&args[0], &args[1], &args[2]);
    args[0].clone()
}

pub fn bcopy(state: &mut State, args: Vec<Value>) -> Value {
    state.memory.memmove(&args[0], &args[1], &args[2]);
    Value::Concrete(0)
}

pub fn bzero(state: &mut State, args: Vec<Value>) -> Value {
    memset(state, vec!(args[0].clone(), Value::Concrete(0), args[1].clone()));
    Value::Concrete(0)
}

pub fn mempcpy(state: &mut State, args: Vec<Value>) -> Value {
    memcpy(state, args.clone()) + args[2].clone()
}

pub fn memccpy(state: &mut State, args: Vec<Value>) -> Value {
    memcpy(state, args)
}

pub fn memfrob(state: &mut State, args: Vec<Value>) -> Value {
    //state.proc.parse_expression( // this is the fun way to do it
    //"0,A1,-,DUP,DUP,?{,A1,-,A0,+,DUP,[1],0x2a,^,SWAP,=[1],1,+,1,GOTO,}", state)
    let addr = &args[0];
    let num = &args[1];

    let x = Value::Concrete(0x2a);
    let data = state.memory.read_sym_len(&addr, &num);
    let mut new_data = vec!();
    for d in data {
        new_data.push(d.clone() ^ x.clone());
    }

    state.memory.write_sym_len(addr, new_data, &num);
    //state.mem_copy(addr, data, num)
    Value::Concrete(0)
}

pub fn strlen(state: &mut State, args: Vec<Value>) -> Value {
    state.memory.strlen(&args[0], &Value::Concrete(MAX_LEN))
}

pub fn strnlen(state: &mut State, args: Vec<Value>) -> Value {
    state.memory.strlen(&args[0], &args[1])
}

// TODO implement this with sim fs
pub fn gets(state: &mut State, args: Vec<Value>) -> Value {
    let bv = state.bv(format!("gets_{:?}", &args[0]).as_str(), 256*8);
    state.memory.write_sym(&args[0], Value::Symbolic(bv), 256);
    args[0].clone()
}

/*pub fn fgets(state: &mut State, addr: Value, length: Value, f: Value):
    fd = fileno(state: &mut State, f)
    read(state: &mut State, BV(fd), addr, length)
    return addr*/

pub fn strcpy(state: &mut State, args: Vec<Value>) -> Value {
    let length = state.memory.strlen(&args[1], &Value::Concrete(MAX_LEN))
        +Value::Concrete(1);
    state.memory.memmove(&args[0], &args[1], &length);
    args[0].clone()
}

pub fn stpcpy(state: &mut State, args: Vec<Value>) -> Value {
    let length = state.memory.strlen(&args[1], &Value::Concrete(MAX_LEN));
    strcpy(state, args) + length
}

pub fn strdup(state: &mut State, args: Vec<Value>) -> Value {
    let length = state.memory.strlen(&args[0], &Value::Concrete(MAX_LEN))
        +Value::Concrete(1);
    let new_addr = malloc(state, vec!(length.clone()));
    state.memory.memmove(&new_addr, &args[0], &length);
    new_addr
}

pub fn strdupa(state: &mut State, args: Vec<Value>) -> Value {
    let length = state.memory.strlen(&args[0], &Value::Concrete(MAX_LEN))
        +Value::Concrete(1);
    strdup(state, args) + length
}

// TODO for strn stuff I may need to add a null?
pub fn strndup(state: &mut State, args: Vec<Value>) -> Value {
    let length = state.memory.strlen(&args[0], &args[1]);
    let new_addr = malloc(state, vec!(length.clone()));
    state.memory.memmove(&new_addr, &args[0], &length);
    new_addr
}

pub fn strndupa(state: &mut State, args: Vec<Value>) -> Value {
    let length = state.memory.strlen(&args[0], &args[1]);
    strndup(state, args) + length
}

pub fn strfry(_state: &mut State, args: Vec<Value>) -> Value {
    /*length, last = state.mem_search(addr, [BZERO])
    data = state.mem_read(addr, length)
    // random.shuffle(data) // i dont actually want to do this?
    state.mem_copy(addr, data, length)*/
    args[0].clone()
}

pub fn strncpy(state: &mut State, args: Vec<Value>) -> Value {
    let length = state.memory.strlen(&args[1], &args[2]);
    state.memory.memmove(&args[0], &args[1], &length);
    args[0].clone()
}

pub fn strcat(state: &mut State, args: Vec<Value>) -> Value {
    let length1 = state.memory.strlen(&args[0], &Value::Concrete(MAX_LEN));
    let length2 = state.memory.strlen(&args[1], &Value::Concrete(MAX_LEN))+Value::Concrete(1);
    state.memory.memmove(&(args[0].clone() + length1), &args[1], &length2);
    args[0].clone()
}

pub fn strncat(state: &mut State, args: Vec<Value>) -> Value {
    let length1 = state.memory.strlen(&args[0], &args[2]);
    let length2 = state.memory.strlen(&args[1], &args[2])+Value::Concrete(1);
    state.memory.memmove(&(args[0].clone() + length1), &args[1], &length2);
    args[0].clone()
}

pub fn memset(state: &mut State, args: Vec<Value>) -> Value {
    let mut data = vec!();
    let length = state.solver.max_value(&args[2]);

    for _ in 0..length {
        data.push(args[1].clone());
    }

    state.memory.write_sym_len(&args[0], data, &args[2]);
    args[0].clone()
}

pub fn memchr_help(state: &mut State, args: Vec<Value>, reverse: bool) -> Value {
    state.memory.search(&args[0], &args[1], &args[2], reverse)
}

pub fn memchr(state: &mut State, args: Vec<Value>) -> Value {
    memchr_help(state, args, false)
}

pub fn memrchr(state: &mut State, args: Vec<Value>) -> Value {
    memchr_help(state, args, true)
}

pub fn strchr_help(state: &mut State, args: Vec<Value>, reverse: bool) -> Value {
    let length = state.memory.strlen(&args[0], &Value::Concrete(MAX_LEN));
    memchr_help(state, vec!(args[0].clone(), args[1].clone(), length), reverse)
}

pub fn strchr(state: &mut State, args: Vec<Value>) -> Value {
    strchr_help(state, args, false)
}

pub fn strrchr(state: &mut State, args: Vec<Value>) -> Value {
    strchr_help(state, args, true)
}

pub fn memcmp(state: &mut State, args: Vec<Value>) -> Value {
    state.memory.compare(&args[0], &args[1], &args[2])
}

pub fn strcmp(state: &mut State, args: Vec<Value>) -> Value {    
    let len1 = state.memory.strlen(&args[0], &Value::Concrete(MAX_LEN));
    let len2 = state.memory.strlen(&args[1], &Value::Concrete(MAX_LEN));
    let length  = state.solver.conditional(&(len1.clone().ult(len2.clone())), 
        &len1, &len2)+Value::Concrete(1);

    state.memory.compare(&args[0], &args[1], &length)
}

pub fn strncmp(state: &mut State, args: Vec<Value>) -> Value {
    let len1 = state.memory.strlen(&args[0], &args[2]);
    let len2 = state.memory.strlen(&args[1], &args[2]);
    let length  = state.solver.conditional(&(len1.clone().ult(len2.clone())), 
        &len1, &len2)+Value::Concrete(1);

    state.memory.compare(&args[0], &args[1], &length)
}

// TODO properly handle sym slens
pub fn memmem(state: &mut State, args: Vec<Value>) -> Value {
    let len = state.solver.min_value(&args[3]) as usize;
    let needle_val = state.memory.read_sym(&args[2], len);
    memchr_help(state, vec!(args[0].clone(), needle_val, args[1].clone()), false)
}

pub fn strstr(state: &mut State, args: Vec<Value>) -> Value {
    let dlen = state.memory.strlen(&args[0], &Value::Concrete(MAX_LEN));
    let slen = state.memory.strlen(&args[1], &Value::Concrete(MAX_LEN));
    let len = state.solver.min_value(&slen) as usize;
    let needle_val = state.memory.read_sym(&args[0], len);
    memchr_help(state, vec!(args[0].clone(), needle_val, dlen), false)
}

pub fn malloc(state: &mut State, args: Vec<Value>) -> Value {
    Value::Concrete(state.memory.alloc(&args[0]))
}

pub fn calloc(state: &mut State, args: Vec<Value>) -> Value {
    Value::Concrete(state.memory.alloc(&(args[0].clone()*args[1].clone())))
}

pub fn free(state: &mut State, args: Vec<Value>) -> Value {
    state.memory.free(&args[0]);
    Value::Concrete(0)
}

/*
pub fn atoi_helper(state: &mut State, addr, size=SIZE): // still sucks
    string, length = state.symbolic_string(addr)

    if z3.is_bv_value(string):
        cstr = state.evaluate_string(string)
        return BV(int(cstr), size)
    else:
        length = state.evalcon(length).as_long() // unfortunate

        result = BV(0, size)
        is_neg = z3.BoolVal(False)
        m = BV(ord("-"), 8)
        for i in range(length):
            d = state.mem_read_bv(addr+i, 1)
            is_neg = z3.If(d == m, z3.BoolVal(True), is_neg)
            c = z3.If(d == m, BV(0, size), z3.ZeroExt(size-8, d-BV_0))
            result = result+(c*BV(10**(length-(i+1)), size))

        result = z3.If(is_neg, -result, result)
        return result

pub fn atoi(state: &mut State, addr):
    return atoi_helper(state: &mut State, addr, 32)

pub fn atol(state: &mut State, addr):
    return atoi_helper(state: &mut State, addr, state.bits)

pub fn atoll(state: &mut State, addr):
    return atoi_helper(state: &mut State, addr, 64)

pub fn digit_to_char(digit):
    if digit < 10:
        return str(digit)

    return chr(ord('a') + digit - 10)

pub fn str_base(number, base):
    if number < 0:
        return '-' + str_base(-number, base)

    (d, m) = divmod(number, base)
    if d > 0:
        return str_base(d, base) + digit_to_char(m)

    return digit_to_char(m)

pub fn bvpow(bv, ex):
    nbv = BV(1, 128)
    for i in range(ex):
        nbv = nbv*bv
    
    return z3.simplify(nbv)

pub fn itoa_helper(state: &mut State, value, string, base, sign=True):
    // ok so whats going on here is... uhh it works
    data = [BZERO]
    nvalue = z3.SignExt(96, z3.Extract(31, 0, value))
    pvalue = z3.ZeroExt(64, value)
    do_neg = z3.And(nvalue < 0, base == 10, z3.BoolVal(sign))
    base = z3.ZeroExt(64, base)
    new_value = z3.If(do_neg, -nvalue, pvalue)
    shift = BV(0, 128)
    for i in range(32):
        d = (new_value % bvpow(base, i+1)) / bvpow(base, i)
        c = z3.Extract(7, 0, d)
        shift = z3.If(c == BZERO, shift+BV(8, 128), BV(0, 128))
        data.append(z3.If(c < 10, c+BV_0, (c-10)+BV_a))

    pbv = z3.Concat(*data)
    szdiff = pbv.size()-shift.size()
    pbv = pbv >> z3.ZeroExt(szdiff, shift)
    nbv = z3.simplify(z3.Concat(pbv, BV(ord("-"),8)))
    pbv = z3.simplify(z3.Concat(BV(0,8), pbv)) // oof
    state.mem_write(string, z3.If(do_neg, nbv, pbv))
        
    return string

pub fn itoa(state: &mut State, value, string, base):
    return itoa_helper(state: &mut State, value, string, base)
*/

pub fn islower(_state: &mut State, args: Vec<Value>) -> Value {
    let c = args[0].clone().slice(7, 0);
    c.ult(Value::Concrete(0x7b)) & !c.ult(Value::Concrete(0x61))
}

pub fn isupper(_state: &mut State, args: Vec<Value>) -> Value {
    let c = args[0].clone().slice(7, 0);
    c.ult(Value::Concrete(0x5b)) & !c.ult(Value::Concrete(0x41))
}

pub fn isalpha(state: &mut State, args: Vec<Value>) -> Value {
    isupper(state, args.clone()) | islower(state, args)
}

pub fn isdigit(_state: &mut State, args: Vec<Value>) -> Value {
    let c = args[0].clone().slice(7, 0);
    c.ult(Value::Concrete(0x3a)) & !c.ult(Value::Concrete(0x30))
}

pub fn isalnum(state: &mut State, args: Vec<Value>) -> Value {
    isalpha(state, args.clone()) | isdigit(state, args)
}

pub fn isblank(_state: &mut State, args: Vec<Value>) -> Value {
    let c = args[0].clone().slice(7, 0);
    c.clone().eq(Value::Concrete(0x20)) | c.eq(Value::Concrete(0x09))
}

pub fn iscntrl(_state: &mut State, args: Vec<Value>) -> Value {
    let c = args[0].clone().slice(7, 0);
    (c.ugte(Value::Concrete(0)) & c.ulte(Value::Concrete(0x1f)))
        | c.eq(Value::Concrete(0x7f))
}

pub fn toupper(state: &mut State, args: Vec<Value>) -> Value {
    let islo = islower(state, args.clone());
    state.solver.conditional(&islo, 
        &(args[0].clone()-Value::Concrete(0x20)), &args[0])
}

pub fn tolower(state: &mut State, args: Vec<Value>) -> Value {
    let isup = isupper(state, args.clone());
    state.solver.conditional(&isup, 
    &(args[0].clone()+Value::Concrete(0x20)), &args[0])
}

pub fn zero(_state: &mut State, _args: Vec<Value>) -> Value {
    Value::Concrete(0)
}

/*pub fn rand(state: &mut State, _args: Vec<Value>) -> Value {
    let mut rng = rand::thread_rng();
    let rn: u64 = rng.gen();
    Value::Symbolic(state.bv(format!("rand_{}", rn), 32))
}

pub fn srand(state: &mut State, _args: Vec<Value>) -> Value {
    //s = state.evaluate(s).as_long()
    //random.seed(s)
    Value::Concrete(1)
}

pub fn abs(state: &mut State, args: Vec<Value>) -> Value {
    state.solver.conditional(i.sext(
        Value::Concrete(32)).slt(Value::Concrete(0)), -i, i)
}

pub fn labs(state: &mut State, args: Vec<Value>) -> Value {
    state.solver.conditional(args[0].clone().slt(Value::Concrete(0)), -args[0], i)
}

pub fn div(state: &mut State, args: Vec<Value>) -> Value {
    let nn = args[0].clone().slice(31, 0);
    let nd = args[1].clone().slice(31, 0);
    nn / nd
}

pub fn ldiv(state: &mut State, n: Value, d: Value) -> Value {
    n / d 
}
*/

pub fn fflush(_state: &mut State, _args: Vec<Value>) -> Value {
    Value::Concrete(0)
}

pub fn getpid(state: &mut State, _args: Vec<Value>) -> Value {
    Value::Concrete(state.pid)
}

// returning a symbolic pid+1 | 0 | -1
// will result in a split state when used to branch
// essentially recreating a fork. pretty cool!
pub fn fork(state: &mut State, _args: Vec<Value>) -> Value {
    let cpid = state.pid+1;
    state.pid = cpid;
    let pid = state.bv(format!("pid_{}", cpid).as_str(), 64);
    pid._eq(&state.bvv(cpid, 64)).or(&pid._eq(&state.bvv(0, 64)))
        .or(&pid._eq(&state.bvv(-1i64 as u64, 64))).assert();

    Value::Symbolic(pid)
}

pub fn getpagesize(_state: &mut State, _args: Vec<Value>) -> Value {
    Value::Concrete(0x1000)
}

pub fn gethostname(state: &mut State, args: Vec<Value>) -> Value {
    let len = state.solver.max_value(&args[1]);
    let bv = state.bv("hostname", 8*len as u32);
    let data = state.memory.unpack(Value::Symbolic(bv), len as usize);
    state.memory.write_sym_len(&args[0], data, &args[1]);
    Value::Concrete(0)
}

/*
pub fn getenv(state: &mut State, addr):
    name, length = state.symbolic_string(addr)
    con_name = state.evaluate_string(name)
    data = state.os.getenv(con_name)

    if data == None:
        return 0
    else:
        val_addr = state.mem_alloc(len(data)+1)
        state.memory[val_addr] = data
        return val_addr
*/

pub fn sleep(_state: &mut State, _args: Vec<Value>) -> Value {
    Value::Concrete(0)
}

/*
pub fn fileno(state: &mut State, f):
    // this isn't how its really done so ima leave this
    addr = state.evalcon(f).as_long()
    bv = state.memory[addr]
    return state.evalcon(bv).as_long()
*/

pub fn open(state: &mut State, args: Vec<Value>) -> Value {
    let addr = state.solver.evalcon_to_u64(&args[0]).unwrap();
    let len = state.memory.strlen(&args[0], &Value::Concrete(MAX_LEN));
    let length = state.solver.evalcon_to_u64(&len).unwrap();
    let path = state.memory.read_string(addr, length as usize);
    if let Some(fd) = state.filesystem.open(path.as_str(), FileMode::Read) {
        Value::Concrete(fd as u64)
    } else {
        Value::Concrete(-1i64 as u64)
    }
}

/*
pub fn mode_to_int(mode):
    m = 0

    if "rw" in mode:
        m |= os.O_RDWR
    elif "r" in mode:
        m |= os.O_RDONLY
    elif "w" in mode:
        m |= os.O_WRONLY
    elif "a" in mode:
        m |= os.O_APPEND

    if "+" in mode:
        m |= os.O_CREAT

    return m


pub fn fopen(state: &mut State, path, mode):
    f = state.mem_alloc(8)
    mode = state.evaluate_string(state.symbolic_string(path)[0])
    flags = mode_to_int(mode)
    fd = open(state: &mut State, path, BV(flags), BV(0o777))
    state.memory[f] = fd
    return f
*/

pub fn close(state: &mut State, args: Vec<Value>) -> Value {
    let fd = state.solver.evalcon_to_u64(&args[0]);
    state.filesystem.close(fd.unwrap() as usize);
    Value::Concrete(0)
}

/*
pub fn fclose(state: &mut State, f):
    fd = fileno(state: &mut State, f)
    return close(state: &mut State, BV(fd))
*/

pub fn read(state: &mut State, args: Vec<Value>) -> Value {
    let fd = state.solver.evalcon_to_u64(&args[0]).unwrap();
    let length = state.solver.max_value(&args[2]);
    let data = state.filesystem.read(fd as usize, length as usize);
    let len = data.len();
    state.memory.write_sym_len(&args[1], data, &args[2]);
    Value::Concrete(len as u64)
}

/*
pub fn fread(state: &mut State, addr, sz, length, f):
    fd = fileno(state: &mut State, f)
    return read(state: &mut State, BV(fd), addr, sz*length)
*/

pub fn write(state: &mut State, args: Vec<Value>) -> Value {
    let fd = state.solver.evalcon_to_u64(&args[0]).unwrap();
    let data = state.memory.read_sym_len(&args[1], &args[2]);
    let len = data.len();
    state.filesystem.write(fd as usize, data);
    Value::Concrete(len as u64)
}

/*
pub fn fwrite(state: &mut State, addr, sz, length, f):
    fd = fileno(state: &mut State, f)
    return write(state: &mut State, BV(fd), addr, sz*length)
*/

pub fn lseek(state: &mut State, args: Vec<Value>) -> Value {
    let fd = state.solver.evalcon_to_u64(&args[0]).unwrap();
    let pos = state.solver.evalcon_to_u64(&args[2]).unwrap();
    state.filesystem.seek(fd as usize, pos as usize);
    Value::Concrete(pos)
}

pub fn access(state: &mut State, args: Vec<Value>) -> Value {
    let addr = state.solver.evalcon_to_u64(&args[0]).unwrap();
    let len = state.memory.strlen(&args[0], &Value::Concrete(MAX_LEN));
    let length = state.solver.evalcon_to_u64(&len).unwrap();
    let path = state.memory.read_string(addr, length as usize);
    state.filesystem.access(path.as_str())
}

pub fn exit(state: &mut State, _args: Vec<Value>) -> Value {
    state.status = StateStatus::Inactive;
    Value::Concrete(0)
}

/*
pub fn fseek(state: &mut State, f, offset, whence):
    fd = fileno(state: &mut State, f)
    return lseek(state: &mut State, BV(fd), offset, whence)


pub fn access(state: &mut State, path, flag): // TODO: complete this
    path = state.symbolic_string(path)[0]
    path = state.evaluate_string(path)
    return state.fs.exists(path)

pub fn stat(state: &mut State, path, data): // TODO: complete this
    path = state.symbolic_string(path)[0]
    path = state.evaluate_string(path)
    return state.fs.exists(path)


pub fn system(state: &mut State, cmd):
    string, length = state.symbolic_string(cmd)
    logger.warning("system(%s)" % state.evaluate_string(string)) // idk
    return 0

pub fn abort(state):
    logger.info("process aborted")
    state.exit = 0
    return 0

pub fn simexit(state: &mut State, status):
    logger.info("process exited")
    state.exit = status
    return 0

pub fn print_stdout(s: str):
    try:
        from colorama import Fore, Style
        sys.stdout.write(Fore.YELLOW+s+Style.RESET_ALL)
    except:
        sys.stdout.write(s)

pub fn nothin(state):
    return 0
    
pub fn ret_one(state):
    return 1

pub fn ret_negone(state):
    return BV(-1)

pub fn ret_arg1(state: &mut State, a):
    return a

pub fn ret_arg2(state: &mut State, a, b):
    return b

pub fn ret_arg3(state: &mut State, a, b, c):
    return c

pub fn ret_arg4(state: &mut State, a, b, c, d):
    return d

UINT = 0
SINT = 1
FLOAT = 2
PTR = 3

pub fn ieee_to_float(endian, v, size=64):
    e = "<"
    if endian == "big":
        e = ">"

    o = e+"d"
    i = e+"Q"
    if size == 32:
        o = e+"f"
        i = e+"I"

    return unpack(o, pack(i, v))[0]

pub fn convert_arg(state: &mut State, arg, typ, size, base):

    szdiff = size-arg.size()

    if szdiff > 0:
        if typ == SINT:
            arg = z3.SignExt(szdiff, arg)
        else:
            arg = z3.ZeroExt(szdiff, arg)
    elif szdiff < 0:
        arg = z3.Extract(size-1, 0, arg)

    arg = state.evalcon(arg)
    if typ == UINT:
        return arg.as_long()
    elif typ == SINT:
        return arg.as_signed_long()
    elif typ == FLOAT:
        argl = arg.as_long()
        return ieee_to_float(state.endian, argl, size)
    else:
        addr = arg.as_long()
        string = state.symbolic_string(addr)[0]
        return state.evaluate_string(string)

// this sucks 
pub fn format_writer(state: &mut State, fmt, vargs):
    fmts = {
        "c":   ["c",  UINT,  8, 10],
        "d":   ["d",  SINT,  32, 10],
        "i":   ["i",  SINT,  32, 10],
        "u":   ["u",  UINT,  32, 10],
        "e":   ["e",  FLOAT, 64, 10],
        "E":   ["E",  FLOAT, 64, 10],
        "f":   ["f",  FLOAT, 32, 10],
        "lf":  ["lf", FLOAT, 64, 10],
        "Lf":  ["Lf", FLOAT, 64, 10],
        "g":   ["g",  FLOAT, 64, 10],
        "G":   ["G",  FLOAT, 64, 10],
        "hi":  ["hi", SINT,  16, 10],
        "hu":  ["hu", UINT,  16, 10],
        "lu":  ["lu", UINT,  state.bits, 10],
        "ld":  ["ld", SINT,  state.bits, 10],
        "li":  ["li", SINT,  state.bits, 10],
        "p":   ["x",  UINT,  state.bits, 16],
        "llu": ["lu", UINT,  64, 10],
        "lld": ["ld", SINT,  64, 10],
        "lli": ["li", SINT,  64, 10],
        "x":   ["x",  UINT,  32, 16],
        "hx":  ["x",  UINT,  16, 16],
        "lx":  ["x",  UINT,  state.bits, 16],
        "llx": ["x",  UINT,  64, 16],
        "o":   ["o",  UINT,  32, 8],
        "s":   ["s",  PTR,   state.bits, 10],
        //"n":   ["",  PTR,   state.bits, 10],
    }

    '''if fmt.count("%") == 1:
        r_str = ""
        p_ind = fmt.index("%")

        i = p_ind+1
        shiftstr = ""
        while not fmt[i].isalpha():
            shiftstr += fmt[i]
            i += 1'''

    new_args = []
    new_fmt = ""

    ind = 0
    argc = 0
    while ind < len(fmt):
        new_fmt += fmt[ind]
        if fmt[ind] != "%":  
            ind += 1
        else:  
            ind += 1
            nextc = fmt[ind:ind+1]
            if nextc == "%":
                new_fmt += nextc

            else:
                arg = vargs[argc]
                argc += 1

                while not nextc.isalpha():
                    new_fmt += nextc
                    ind += 1
                    nextc = fmt[ind:ind+1]
                
                next3fmt = fmt[ind:ind+3]
                next2fmt = fmt[ind:ind+2]
                next1fmt = fmt[ind:ind+1]

                if next3fmt in fmts:
                    rep, typ, sz, base = fmts[next3fmt]
                    new_args += [convert_arg(state: &mut State, arg, typ, sz, base)]
                    new_fmt += rep
                    ind += 3

                elif next2fmt in fmts:
                    rep, typ, sz, base = fmts[next2fmt]
                    new_args += [convert_arg(state: &mut State, arg, typ, sz, base)]
                    new_fmt += rep
                    ind += 2

                elif next1fmt in fmts:
                    rep, typ, sz, base = fmts[next1fmt]
                    new_args += [convert_arg(state: &mut State, arg, typ, sz, base)]
                    new_fmt += rep
                    ind += 1
                
                elif next1fmt == "n":
                    lastind = len(new_fmt)-new_fmt[::-1].index("%")-1
                    n = len(new_fmt[:lastind]%tuple(new_args))
                    state.mem_write(arg, n)

    return new_fmt % tuple(new_args)

*/

