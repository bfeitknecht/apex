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
  function selection(){
    showCursor();
    highlights.replaceChildren();
    const s=window.getSelection();
    if(!s || s.isCollapsed || !s.rangeCount || !layer.contains(s.anchorNode) || !layer.contains(s.focusNode)) return;
    for(const r of s.getRangeAt(0).getClientRects()){
      if(r.width<=0 || r.height<=0) continue;
      const d=document.createElement('div');d.className='selection-rect';
      Object.assign(d.style,{left:r.left+'px',top:r.top+'px',width:r.width+'px',height:r.height+'px'});
      highlights.appendChild(d);
    }
  }
  document.addEventListener('selectionchange',selection);
  window.addEventListener('resize',selection);
})();
