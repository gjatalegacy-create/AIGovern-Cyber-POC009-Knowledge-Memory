"""Render the research manuscript; requires reportlab and an explicit TrueType font."""
import argparse, html, re
from pathlib import Path
from reportlab.lib import colors
from reportlab.lib.styles import getSampleStyleSheet, ParagraphStyle
from reportlab.lib.enums import TA_LEFT
from reportlab.lib.pagesizes import A4
from reportlab.pdfbase import pdfmetrics
from reportlab.pdfbase.ttfonts import TTFont
from reportlab.platypus import SimpleDocTemplate, Paragraph, Table, TableStyle, Preformatted

parser=argparse.ArgumentParser()
parser.add_argument('--font',required=True)
parser.add_argument('--bold-font',required=True)
args=parser.parse_args()
root=Path(__file__).resolve().parents[1]
pdfmetrics.registerFont(TTFont('Body',args.font))
pdfmetrics.registerFont(TTFont('BodyBold',args.bold_font))
pdfmetrics.registerFontFamily('Body',normal='Body',bold='BodyBold',italic='Body',boldItalic='BodyBold')
styles=getSampleStyleSheet()
styles.add(ParagraphStyle(name='Text',fontName='Body',fontSize=10.2,leading=14.5,spaceAfter=8,textColor=colors.HexColor('#172536')))
styles.add(ParagraphStyle(name='Top',fontName='BodyBold',fontSize=19,leading=23,spaceAfter=14,textColor=colors.HexColor('#102d4c')))
styles.add(ParagraphStyle(name='Section',fontName='BodyBold',fontSize=12.4,leading=16,spaceBefore=13,spaceAfter=8,keepWithNext=True,textColor=colors.HexColor('#102d4c')))
styles.add(ParagraphStyle(name='Eq',fontName='Courier',fontSize=9,leading=13,leftIndent=12,spaceBefore=8,spaceAfter=10))
def markup(text):
    text=text.replace('—','-').replace('–','-')
    text=re.sub(r'\\\((.*?)\\\)',r'\1',text)
    text=html.escape(text)
    text=re.sub(r'\[([^\]]+)\]\((https?://[^)]+)\)',r'<link href="\2" color="#14608c">\1</link>',text)
    text=re.sub(r'\*\*(.*?)\*\*',r'<b>\1</b>',text)
    text=re.sub(r'(?<!\*)\*([^*]+)\*(?!\*)',r'<i>\1</i>',text)
    text=re.sub(r'`([^`]+)`',r'<font name="Courier" size="9">\1</font>',text)
    return text
text=(root/'paper/manuscript.md').read_text(encoding='utf-8')
story=[]
for block in re.split(r'\n\s*\n',text):
    block=block.strip()
    if not block: continue
    if block.startswith('|'):
        rows=[line.strip().strip('|').split('|') for line in block.splitlines() if not re.match(r'^\|[\s:|\-]+$',line)]
        table=Table([[Paragraph(markup(cell.strip()),styles['Text']) for cell in row] for row in rows],colWidths=[42,160,297],repeatRows=1)
        table.setStyle(TableStyle([('BACKGROUND',(0,0),(-1,0),colors.HexColor('#e8f0f6')),('VALIGN',(0,0),(-1,-1),'TOP'),('LINEBELOW',(0,0),(-1,-1),0.3,colors.HexColor('#c8d7e2')),('TOPPADDING',(0,0),(-1,-1),5),('BOTTOMPADDING',(0,0),(-1,-1),5)]))
        story.append(table)
    elif block.startswith('# '): story.append(Paragraph(markup(block[2:]),styles['Top']))
    elif block.startswith('## '): story.append(Paragraph(markup(block[3:]),styles['Section']))
    elif block.startswith('EQUATION: '): story.append(Preformatted(block[10:],styles['Eq']))
    elif re.match(r'^\d+\. ',block):
        for item in re.split(r'\n(?=\d+\. )',block):
            story.append(Paragraph(markup(item.replace('\n',' ')),styles['Text']))
    else:
        p=Paragraph(markup(block.replace('\n',' ')),styles['Text'])
        if block.endswith('by') or block.endswith('is'): p.keepWithNext=True
        story.append(p)
def page(canvas,doc):
    canvas.setStrokeColor(colors.HexColor('#c8d7e2')); canvas.line(48,38,A4[0]-48,38)
    canvas.setFont('Body',8); canvas.setFillColor(colors.HexColor('#52677b'))
    canvas.drawString(48,26,'ESS-MAI / AIGovern-Cyber POC009 | Research preprint | 24 October 2026')
    canvas.drawRightString(A4[0]-48,26,str(doc.page))
output=root/'paper/scoped_knowledge_memory.pdf'
SimpleDocTemplate(str(output),pagesize=A4,rightMargin=48,leftMargin=48,topMargin=45,bottomMargin=53,title='Scoped Model Support and Counterevidence Across Five Memory Cycles in ESS-MAI',author='Bledar Gjata').build(story,onFirstPage=page,onLaterPages=page)
print(output)
