import zipfile,struct,sys
with zipfile.ZipFile(sys.argv[1]) as z:
 d=z.read('classes11.dex'); u=lambda o:struct.unpack_from('<I',d,o)[0];so=u(60);to=u(68)
 def s(i):
  o=u(so+4*i)
  while d[o]&128:o+=1
  o+=1
  return d[o:d.index(0,o)].decode()
 def t(i):return s(u(to+4*i))
 for j in range(u(96)):
  o=u(100)+32*j;n=t(u(o))
  if n in ['LX/Mvc;','LX/Nbs;','LX/Ncy;']:
   it=u(o+12);print(n,'super',t(u(o+8)),'interfaces',[t(struct.unpack_from('<H',d,it+4+2*k)[0]) for k in range(u(it))] if it else [])
