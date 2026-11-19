/* Cell cursor and selection highlights drawn over the rendered canvas. */
(()=>{
  const layer=document.getElementById('selectable-text');
  const cursor=document.getElementById('cell-cursor');
  const highlights=document.getElementById('selection-highlights');
  let point=null, mouseMode=false, queued=false;
  function showCursor(){
    queued=false;
    if(!point || !mouseMode || (window.getSelection() && !window.getSelection().isCollapsed) || (getComputedStyle(layer).display!=='none' && layer.dataset.clickable==='true')) { cursor.style.display='none'; return; }
    const s=getComputedStyle(layer), cw=parseFloat(s.getPropertyValue('--cell-width')), ch=parseFloat(s.getPropertyValue('--cell-height'));
    if(!cw || !ch){cursor.style.display='none'; return;}
    cursor.style.width=cw+'px';cursor.style.height=ch+'px';
    cursor.style.left=Math.floor(point.x/cw)*cw+'px';cursor.style.top=Math.floor(point.y/ch)*ch+'px';cursor.style.display='block';
  }
  document.addEventListener('mousemove',e=>{point={x:e.clientX,y:e.clientY};mouseMode=true;if(!queued){queued=true;requestAnimationFrame(showCursor)}});
  document.addEventListener('keydown',e=>{if(!['Shift','Control','Alt','Meta'].includes(e.key)){mouseMode=false;cursor.style.display='none';layer.dataset.clickable='false';}});
  document.addEventListener('wheel',()=>{mouseMode=false;cursor.style.display='none';},{passive:true});
  document.addEventListener('mouseleave',()=>{mouseMode=false;cursor.style.display='none';});
  // One highlight per row, snapped to whole cells. The browser returns several
  // rects per row with fractional edges; under an inverting backdrop filter an
  // overlap inverts twice and a gap not at all, which drew thin vertical lines.
  function selection(){
    showCursor();
    highlights.replaceChildren();
    const s=window.getSelection();
    if(!s || s.isCollapsed || !s.rangeCount || !layer.contains(s.anchorNode) || !layer.contains(s.focusNode)) return;
    const st=getComputedStyle(layer), cw=parseFloat(st.getPropertyValue('--cell-width')), ch=parseFloat(st.getPropertyValue('--cell-height'));
    if(!cw || !ch) return;
    const rows=new Map();
    for(const r of s.getRangeAt(0).getClientRects()){
      if(r.width<=0 || r.height<=0) continue;
      const row=Math.floor((r.top+r.height/2)/ch), a=Math.round(r.left/cw), b=Math.round(r.right/cw);
      if(b<=a) continue;
      const span=rows.get(row);
      rows.set(row, span ? [Math.min(span[0],a), Math.max(span[1],b)] : [a,b]);
    }
    for(const [row,[a,b]] of rows){
      const d=document.createElement('div');d.className='selection-rect';
      Object.assign(d.style,{left:a*cw+'px',top:row*ch+'px',width:(b-a)*cw+'px',height:ch+'px'});
      highlights.appendChild(d);
    }
  }
  document.addEventListener('selectionchange',selection);
  window.addEventListener('resize',selection);
})();
