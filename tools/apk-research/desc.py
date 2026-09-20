#!/usr/bin/env python3
"""desc.py DUMP CLASS [CLASS...] | --anchor STR...  : parse protobuf-lite RawMessageInfo descriptors (dynamicMethod)."""
import sys,re
path=sys.argv[1]; args=sys.argv[2:]; anchors=[]; classes=[]
i=0
while i<len(args):
    if args[i]=='--anchor': anchors.append(args[i+1]); i+=2
    else: classes.append(args[i]); i+=1
def unesc(s): return re.sub(r'\\u\{([0-9a-fA-F]+)\}', lambda x: chr(int(x.group(1),16)), s).replace('\\0','\0')
def rd(d,idx):
    c=ord(d[idx]); idx+=1
    if c>=0xD800:
        r=c&0x1FFF; sh=13
        while True:
            c=ord(d[idx]); idx+=1
            if c>=0xD800: r|=(c&0x1FFF)<<sh; sh+=13
            else: r|=c<<sh; return r,idx
    return c,idx
TYPES={0:'double',1:'float',2:'int64',3:'uint64',4:'int32',5:'fixed64',6:'fixed32',7:'bool',8:'string',9:'message',10:'bytes',11:'uint32',12:'enum',13:'sfixed32',14:'sfixed64',15:'sint32',16:'sint64',17:'group',18:'double_list',19:'float_list',20:'int64_list',21:'uint64_list',22:'int32_list',23:'fixed64_list',24:'fixed32_list',25:'bool_list',26:'string_list',27:'message_list',28:'bytes_list',29:'uint32_list',30:'enum_list',31:'sfixed32_list',32:'sfixed64_list',33:'sint32_list',34:'sint64_list',35:'double_list_packed',36:'float_list_packed',37:'int64_list_packed',38:'uint64_list_packed',39:'int32_list_packed',40:'fixed64_list_packed',41:'fixed32_list_packed',42:'bool_list_packed',43:'uint32_list_packed',44:'enum_list_packed',45:'sfixed32_list_packed',46:'sfixed64_list_packed',47:'sint32_list_packed',48:'sint64_list_packed',49:'group_list',50:'map',51:'oneof_double',52:'oneof_float',53:'oneof_int64',54:'oneof_uint64',55:'oneof_int32',56:'oneof_fixed64',57:'oneof_fixed32',58:'oneof_bool',59:'oneof_string',60:'oneof_message',61:'oneof_bytes',62:'oneof_uint32',63:'oneof_enum',64:'oneof_sfixed32',65:'oneof_sfixed64',66:'oneof_sint32',67:'oneof_sint64',68:'oneof_group'}
def parse(names, desc):
    d=unesc(desc); idx=0
    flags,idx=rd(d,idx); fc,idx=rd(d,idx)
    if fc==0: return []
    oneofs,idx=rd(d,idx); hasbits,idx=rd(d,idx); mn,idx=rd(d,idx); mx,idx=rd(d,idx); ne,idx=rd(d,idx); mapc,idx=rd(d,idx); repc,idx=rd(d,idx); chk,idx=rd(d,idx)
    for _ in range(chk): _,idx=rd(d,idx)
    res=[]
    # names list: first 2*oneofs objects are oneof case/value field names; skip them
    fnames=names[2*oneofs:]
    ni=0
    while idx < len(d):
        fno,idx=rd(d,idx); ft,idx=rd(d,idx); t=ft&0xff
        if 51<=t<=68:
            oi,idx=rd(d,idx); name=names[2*oi] if 2*oi<len(names) else '?oneof'
        else:
            name=fnames[ni] if ni<len(fnames) else '?'; ni+=1
            if ft&0x1000: _,idx=rd(d,idx)
        res.append((name, fno, TYPES.get(t,str(t))))
    return res
def want(cls, strs):
    if cls in classes: return True
    return any(a in strs for a in anchors)
cls=None; buf=None
with open(path,encoding='utf-8',errors='replace') as fh:
    for line in fh:
        if line.startswith('# class:'): cls=line.strip()[9:-1]; buf=None; continue
        if line.startswith('.method'): buf=[line] if 'dynamicMethod' in line else None; continue
        if buf is None: continue
        buf.append(line)
        if line.startswith('.end method'):
            strs=re.findall(r'const-string(?:/jumbo)?\s+v\d+, "(.*?)"\s*$', "".join(buf), re.M)
            if want(cls,set(strs)):
                desc=[s for s in strs if '\\u{' in s or '\\0' in s]
                if not desc and strs: desc=[strs[-1]]
                names=[s for s in strs if s not in desc and s.endswith('_') and not s.startswith('bitField')]
                if desc:
                    try: print(cls, parse(names, desc[-1]))
                    except Exception as e: print(cls, 'PARSE-ERROR', e, repr(desc[-1])[:120])
            buf=None
