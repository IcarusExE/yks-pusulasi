#!/usr/bin/env python3
"""Rebuild the unit/topic index from the pinned 2026 MEB PDF (requires PyMuPDF).
Usage: python scripts/extract-curriculum.py source.pdf data/curriculum-2026.json
The index is at unit/topic granularity; the PDF remains authoritative for outcomes.
"""
import sys, json, re, hashlib
import fitz
from pathlib import Path
source=Path(sys.argv[1]); doc=fitz.open(source)
assert len(doc)==581
sha=hashlib.sha256(source.read_bytes()).hexdigest()
assert sha=='f52502743b6372e5df7de42d741b02a0b18eec087eac785b50a1fe3dd4a6d536', 'Source changed: review before rebuilding catalogue'
# Encoding repair is restricted to the three legacy-font sections, not arbitrary text.
tr={'Õ':'ı','÷':'ğ','ú':'ş','ù':'Ş','ø':'İ','ö':'Ğ','o':'ç','g':'Ö','|':'ö','\x81':'ü','h':'Ü','d':'Ç','k':'â','v':'î'}
def decode(t):return ''.join(tr.get(c,chr(ord(c)+29) if 3<=ord(c)<=94 else c) for c in t)
rows=[]
for pi,p in enumerate(doc):
 for b in p.get_text('dict')['blocks']:
  for l in b.get('lines',[]):
   text=''.join(s['text'] for s in l['spans'])
   if 239<=pi<271 and '\x03' in text:text=decode(text)
   rows.append(dict(page=pi+1,text=re.sub(r'\s+',' ',text).strip(),font=l['spans'][0]['font'],x=l['bbox'][0],y=l['bbox'][1]))
ranges=[('literature','Türk Dili ve Edebiyatı',4,50),('religion','Din Kültürü ve Ahlak Bilgisi',51,74),('history','Tarih',75,102),('revolution','T.C. İnkılap Tarihi ve Atatürkçülük',103,111),('geography','Coğrafya',112,123),('math','Matematik',124,154),('physics','Fizik',155,182),('chemistry','Kimya',183,206),('biology','Biyoloji',207,226),('philosophy','Felsefe',227,238),('logic','Mantık',239,246),('sociology','Sosyoloji',247,257),('psychology','Psikoloji',258,270),('english','İngilizce',271,316),('german','Almanca',317,348),('french','Fransızca',349,380),('arabic','Arapça',381,501),('russian','Rusça',502,581)]
items=[]
def add(course,title,page,level='Ortak',code=''):
 title=re.sub(r'\s+',' ',title).strip()
 if not title or any(ord(c)<32 for c in title):raise ValueError((course,title,page))
 if any(t['course']==course and t['page']==page and t['title']==title and t['level']==level for t in items):return
 items.append(dict(id=f'{course}-{page}-{len(items)+1}',course=course,title=title,page=page,level=level,code=code))
for course,name,start,end in ranges:
 rs=[r for r in rows if start<=r['page']<=end]
 if course in ['math','physics','chemistry','biology']:
  # Leaf topic headings: 9.x.x and TD.11/12.x.x; outcome codes have one more part.
  headings={}
  for i,r in enumerate(rs):
   m=re.match(r'^((?:TD\.)?(?:9|10|11|12)\.\d+\.\d+)\.?\s*([^\d.\s].*)$',r['text'])
   if not m:continue
   code,title=m.groups()
   # One extraneous duplicated physics heading occurs at p165; the proper section is p173.
   if course=='physics' and code=='11.2.4' and r['page']==165:continue
   headings[code]=(title,r['page'])
  for code,(title,page) in headings.items():add(course,title,page,('Temel düzey · ' if code.startswith('TD.') else '')+re.search(r'(9|10|11|12)\.',code)[1]+'. sınıf',code)
 elif course=='religion':
  for r in rs:
   m=re.match(r'^((?:9|10|11|12)\.\d+)\.\s*(?!\d)(.+)',r['text'])
   if m:add(course,m[2],r['page'],m[1].split('.')[0]+'. sınıf',m[1])
 elif course in ['literature','history','revolution']:
  for r in rs:
   m=re.match(r'^(\d+)\.?\s*ÜN[Iİ]TE\s*:\s*(.+)',r['text'])
   if m:
    page=r['page'];grade=(9 if page<25 else 10 if page<34 else 11 if page<43 else 12) if course=='literature' else ((9 if page<83 else 10 if page<90 else 11) if course=='history' else 12)
    add(course,m[2],page,f'{grade}. sınıf',f'{grade}.{m[1]}')
  if course=='literature':
   for title,page in [('Şiir: okuma kazanımları',5),('Öyküleyici edebî metinler: okuma kazanımları',7),('Tiyatro: okuma kazanımları',9),('Bilgilendirici metinler: okuma kazanımları',10),('Yazma',12),('Sözlü iletişim: konuşma',13),('Sözlü iletişim: dinleme',15)]:add(course,title,page,'Ortak okuma / yazma / sözlü iletişim')
 elif course=='geography':
  labels={1:'Doğal Sistemler',2:'Beşerî Sistemler',3:'Küresel Ortam: Bölgeler ve Ülkeler',4:'Çevre ve Toplum'};seen=set()
  for r in rs:
   m=re.match(r'^(9|10|11|12)\.([1-4])\.\d+\.',r['text'])
   if m and (m[1],m[2]) not in seen:
    seen.add((m[1],m[2]));add(course,labels[int(m[2])],r['page'],m[1]+'. sınıf',m[1]+'.'+m[2])
 elif course in ['philosophy','logic','sociology']:
  for r in rs:
   text=r['text'].replace('=',' ')
   if course=='philosophy':m=re.match(r'^ÜNİTE\s*(\d+)\s*:\s*(.+)',text)
   elif course=='logic':m=re.match(r'^ÜNİTE\s*(\d+)\s*:\s*(.+)',text)
   else:m=re.match(r'^(\d+)\.?\s*ÜNİTE\s*:?\s*(.+)',text)
   if m:add(course,m[2],r['page'],('10. sınıf' if r['page']<233 else '11. sınıf') if course=='philosophy' else 'Ortak',m[1])
 elif course=='psychology':
  # Checked against the rendered PDF; the source's legacy font corrupts these four headings.
  for title,page in [('Psikoloji Bilimini Tanıyalım',259),('Psikolojinin Temel Süreçleri',261),('Öğrenme, Bellek ve Düşünme',265),('Ruh Sağlığının Temelleri',267)]:add(course,title,page)
 elif course=='english':
  grade=''
  for r in rs:
   if m:=re.match(r'(9|10|11|12)th Grade',r['text']):grade=m[1]+'. sınıf'
   if m:=re.match(r'THEME\s*(\d+):\s*(.+)',r['text']):add(course,m[2],r['page'],grade,m[1])
 elif course=='german':
  for r in rs:
   if m:=re.match(r'Themenkreis\s*(\d+):\s*(.+)',r['text']):
    page=r['page'];level='A1.1' if page<=326 else 'A1.2' if page<=334 else 'A2.1' if page<=341 else 'A2.2';add(course,m[2],page,level,m[1])
 elif course=='french':
  level=''
  for r in rs:
   if m:=re.match(r'NIVEAU\s*(A[12]\.\d)',r['text']):level=m[1]
   if m:=re.match(r'THÈME\s*(\d+)\s*:\s*(.+)',r['text']):add(course,m[2],r['page'],level,m[1])
 elif course=='arabic':
  # This curriculum uses grade/level skill blocks instead of numbered themes.
  for r in rs:
   if m:=re.match(r'(9|10|11|12)\. SINIF KAZANIMLARI\s*[(*]',r['text']):
    page=r['page'];branch='Başlangıç A1' if page<418 else 'Başlangıç A2' if page<455 else 'Başlangıç B1'
    notes=' '.join(t['text'] for t in rs if t['page']==page and '**' in t['text']);code=re.search(r'\b[AB][12]\.\d\.\d+\b',notes);code=code[0] if code else 'Genel'
    for skill in ['Dinleme','Karşılıklı konuşma','Sözlü anlatım','Okuma','Yazma']:add(course,skill,page,f'{branch} · {m[1]}. sınıf · {code}',code)
  for page,level in [(493,'A1 ortak'),(494,'A2 ortak'),(495,'B1 ortak')]:
   for skill in ['Dinleme','Konuşma','Okuma','Yazma']:add(course,skill,page,level)
  # Crop the structural left column, not the explanatory Arabic examples.
  for page in range(496,501):
   p=doc[page-1]
   for block in p.get_text('blocks',clip=fitz.Rect(45,310 if page==496 else 50,220,p.rect.height-35)):
    text=block[4].replace('-\n','')
    text=text.replace('Geçmiş zamanı olumsuzlayan\nformlar','Geçmiş zamanı olumsuzlayan formlar')
    for title in text.splitlines():
     title=title.strip()
     if len(title)<3 or title=='Yapılar' or '\uf0b7' in title:continue
     y=block[1];level='A1' if page==496 or (page==497 and y<372) else 'A2' if page==497 or (page==498 and y<295) else 'B1'
     add(course,title,page,level+' · dil yapıları')
 elif course=='russian':
  # Original Cyrillic theme headings; joined columns at p513/p572 are corrected.
  groups=[(9,[(506,'Вводный урок'),(506,'Человек и Знакомство'),(508,'СЕМЬЯ'),(510,'Мой родной город'),(512,'Глаголы движения'),(513,'Мой рабочий день'),(515,'Ваша деятельность'),(517,'Планы и желания'),(518,'Дом'),(519,'Приказы, просьбы и пожелания')]),(10,[(528,'Еда'),(528,'В ресторане'),(531,'Раньше и сейчас'),(531,'Детективный рассказ'),(532,'Известные люди'),(536,'Здоровье'),(536,'Спорт'),(539,'Биография'),(539,'Климат')]),(11,[(546,'Культура'),(546,'Искусство'),(547,'Кино'),(547,'Театр'),(548,'Музыка и танец'),(554,'Досуг'),(554,'В гостях'),(555,'Новости'),(555,'Транспорт'),(556,'Гостиница')]),(12,[(564,'Университеты'),(564,'Великие люди'),(565,'Страны и их культуры'),(565,'Всѐ о Турции'),(566,'Всѐ о России'),(572,'Флора и фауна'),(573,'Будущее нашей планеты'),(574,'Международные и национальные праздники'),(575,'Чудеса света'),(576,'Туризм как путь к познанию другой культуры')])]
  for grade,themes in groups:
   for page,title in themes:add(course,title,page,f'{grade}. sınıf')
  for grade,pages in [(9,[503,504,504,505]),(10,[526,526,527,527]),(11,[544,544,545,545]),(12,[562,562,563,563])]:
   for skill,page in zip(['Dinleme','Konuşma','Okuma','Yazma'],pages):add(course,skill+' becerisi',page,f'{grade}. sınıf · ortak beceriler')
# Source order within each course. IDs depend only on stable record identity, not display order.
for item in items:item['id']=item['course']+'-'+hashlib.sha256((item['level']+'|'+item['title']+'|'+str(item['page'])).encode()).hexdigest()[:12]
items.sort(key=lambda t:([x[0] for x in ranges].index(t['course']),t['page']))
output={'year':2026,'checked_on':'2026-11-22','source_url':'https://ttkb.meb.gov.tr/meb_iys_dosyalar/2025_11/26164023_2026_yks.pdf','source_sha256':sha,'source_pages':581,'granularity':'Ünite, tema ve alt konu; açıklamalar için resmî belge esas alınır.','courses':[dict(id=c,name=n,start_page=a,end_page=b) for c,n,a,b in ranges],'topics':items}
Path(sys.argv[2]).write_text(json.dumps(output,ensure_ascii=False,indent=2)+'\n')
for c,n,a,b in ranges:print(n,len([t for t in items if t['course']==c]))
print('TOTAL',len(items))
