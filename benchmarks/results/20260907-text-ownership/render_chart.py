"""Offline plot from the sealed summary; no new measurements."""
from pathlib import Path
import json,sys,os,tempfile
os.environ.setdefault("MPLCONFIGDIR",str(Path(tempfile.gettempdir())/"anki-forge-plot-cache"))
os.environ.setdefault("XDG_CACHE_HOME",str(Path(tempfile.gettempdir())/"anki-forge-font-cache"))
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
from matplotlib.patches import Patch
D=Path(sys.argv[1]) if len(sys.argv)>1 else Path(__file__).resolve().parent
s=json.loads((D/'summary.json').read_text()); rows={c['name']:c for c in s['cases']}
names=['text-10000','long-back-1000','long-back-quadruple-1000']
labels=['10,000 text notes','1,000 long-field notes\n6.6 MiB of input fields','1,000 long-field notes\n26.1 MiB of input fields']
colors={'current':'#8899A6','accepted':'#187A72'}
plt.rcParams.update({'font.family':'DejaVu Sans','font.size':10,'svg.fonttype':'none','axes.spines.top':False,'axes.spines.right':False,'axes.spines.left':False,'axes.spines.bottom':False})
fig,axes=plt.subplots(1,2,figsize=(12,5.2),sharey=True)
fig.subplots_adjust(left=.245,right=.975,top=.70,bottom=.15,wspace=.25)
for ax,metric,title,limit in zip(axes,['rss_mib','time_ms'],['Peak RSS (MiB)','Export time (ms)'],[197,665]):
    for i,name in enumerate(names):
        c=rows[name]
        for v,offset in [('current',-.17),('accepted',.17)]:
            t=c['variants'][v][metric]; y=i+offset; val=t['median']
            ax.barh(y,val,height=.28,color=colors[v],zorder=3)
            ax.errorbar(val,y,xerr=[[val-t['q1']],[t['q3']-val]],fmt='none',color='#243947',capsize=2,linewidth=.85,zorder=4)
            change=c['rss_change_pct' if metric=='rss_mib' else 'time_change_pct']
            text=f'{val:.1f}'+(f' ({change:+.1f}%)' if v=='accepted' else '')
            ax.text(max(val,t['q3'])+limit*.02,y,text,va='center',fontsize=9.5,color='#172F3E')
    ax.set_xlim(0,limit); ax.set_title(title,loc='left',fontsize=12,fontweight='bold',pad=14)
    ax.set_yticks(range(3),labels); ax.tick_params(axis='y',length=0,pad=15); ax.tick_params(axis='x',length=0,labelcolor='#536675')
    ax.xaxis.grid(True,color='#E8EDF0',zorder=0)
    ax.set_ylim(2.55,-.55)
fig.text(.035,.93,'Text ownership reduces export memory',fontsize=18,fontweight='bold',color='#172F3E')
fig.text(.035,.88,'Same inputs and exact APKG bytes · 7 timings + 5 separate RSS samples per version · AC power',fontsize=10.5,color='#536675')
fig.legend(handles=[Patch(color=colors['current'],label='Previous shared-buffer build'),Patch(color=colors['accepted'],label='Consume temporary Project')],loc='upper left',bbox_to_anchor=(.027,.846),frameon=False,ncol=2)
fig.text(.035,.045,'Medians; whiskers show Q1–Q3 sample spread. Rust release/default features, system allocator. Synthetic workloads.',fontsize=9,color='#536675')
fig.savefig(D/'text-ownership.svg',facecolor='white')
fig.savefig(D/'text-ownership.png',dpi=150,facecolor='white')
