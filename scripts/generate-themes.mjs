// Regenerates the CLIPLY_THEMES block in src/theme/theme.ts.
//
// Every ramp (hover, active, wash, border, on-soft, button text) is derived
// from the accent and checked against WCAG, so adding a theme means adding one
// line to the `defs` table rather than hand-picking a dozen hex values and
// hoping they pass contrast.
//
// Usage: node scripts/generate-themes.mjs
const lin=c=>{c/=255;return c<=0.04045?c/12.92:Math.pow((c+0.055)/1.055,2.4)};
const hx=h=>[1,3,5].map(i=>parseInt(h.slice(i,i+2),16));
const L=h=>{const [r,g,b]=hx(h);return 0.2126*lin(r)+0.7152*lin(g)+0.0722*lin(b)};
const R=(a,b)=>{const l1=Math.max(L(a),L(b)),l2=Math.min(L(a),L(b));return (l1+0.05)/(l2+0.05)};
const mix=(a,b,t)=>{const A=hx(a),B=hx(b);return '#'+A.map((v,i)=>Math.round(v+(B[i]-v)*t).toString(16).padStart(2,'0')).join('').toUpperCase()};
const INK='#14161A';
const onSoft=(base,soft)=>{for(let t=0;t<=0.8;t+=0.02){const c=mix(base,'#000000',t);if(R(c,soft)>=4.6)return c}return mix(base,'#000000',0.8)};

const defs=[
  ['coral-pulse','珊瑚跃动','与 Cliply 图标同源的明快珊瑚色，轻盈、有活力，默认主题。','#FF6257'],
  ['system-blue','晴空蓝','清透明快的晴空蓝，年轻、清晰，默认主题。','#2F69FA'],
  ['lake-blue','湖蓝','饱和的宝蓝，清晰有力。','#1D5FD6'],
  ['indigo-spark','电光靛','冷冽的电光靛蓝，年轻、有速度感。','#4F46E5'],
  ['purple-default','霓虹紫','明快的紫色，创意工具的气质。','#6D4CFF'],
  ['magenta-pop','荧光洋红','高饱和洋红，最张扬的一款。','#D6218C'],
  ['rose-violet','玫红','明亮的玫红，柔中带锐。','#DB2777'],
  ['coral-orange','珊瑚橙','热烈的珊瑚橙，暖而醒目。','#E8552D'],
  ['amber-glow','暖阳金','温暖的琥珀金，明亮不刺眼。','#C2820A'],
  ['lime-punch','青柠绿','鲜亮的青柠绿，轻快有生气。','#4E9F0D'],
  ['mint-green','薄荷绿','清新的薄荷绿，自然舒展。','#1BA36B'],
  ['teal-fresh','清爽青','沉稳的青绿，专业感更强。','#0D9488'],
];

// The brand coral stays luminous across interaction states. A large black
// mix made this hue look muddy, so hover lifts and active only settles a touch.
const stateOverrides = {
  'coral-pulse': { hover: '#FF7066', active: '#F75A50' },
};

const NEUTRAL = {
  appBg:'#F5F9FD', windowBg:'#F5F9FD', panelBg:'#FFFFFF', cardBg:'#FFFFFF',
  inputBg:'#FFFFFF', mutedBg:'#EFF5FA', border:'#E2EAF2', borderStrong:'#CEDBE8',
  divider:'#EDF3F8', text:'#1B2734', bodyText:'#2C3A49', textSecondary:'#5F6F80',
  muted:'#64748B', placeholder:'#7C8B9C', disabledText:'#B9C5D1',
  success:'#168F73', successSoft:'#E8F7F2', warning:'#B45309', warningSoft:'#FFF7E6',
  danger:'#DC2626', dangerSoft:'#FEF2F2',
};

let out = 'export const CLIPLY_THEMES: Record<CliplyThemeName, CliplyThemeTokens> = {\n';
const report=[];
for (const [name,label,description,primary] of defs) {
  // Derive the wash so the row caption (secondary text) still clears 4.5:1
  // on top of it — a fixed mix ratio fails for saturated hues.
  let soft = mix(primary,'#FFFFFF',0.90);
  for (let t=0.90; t<=0.97; t+=0.01) { soft = mix(primary,'#FFFFFF',t); if (R('#5F6F80', soft) >= 4.55) break; }
  const border = mix(primary,'#FFFFFF',0.75);
  const text = R(primary,'#FFFFFF')>=4.5 ? '#FFFFFF' : INK;
  const hover = stateOverrides[name]?.hover ?? (text === INK ? mix(primary,'#FFFFFF',0.08) : mix(primary,'#000000',0.12));
  const active = stateOverrides[name]?.active ?? (text === INK ? mix(primary,'#000000',0.04) : mix(primary,'#000000',0.22));
  const os = onSoft(primary,soft);
  const [r,g,b]=hx(primary);
  report.push({name, btn:+R(primary,text).toFixed(2), hover:+R(hover,text).toFixed(2), active:+R(active,text).toFixed(2), wash:+R(os,soft).toFixed(2), rail:+R(primary,'#FFFFFF').toFixed(2), cap:+R('#5F6F80',soft).toFixed(2), text:text==='#FFFFFF'?'white':'ink'});
  out += `  "${name}": {
    name: "${name}",
    label: "${label}",
    description: "${description}",

    primary: "${primary}",
    primaryHover: "${hover}",
    primaryActive: "${active}",
    primarySoft: "${soft}",
    primaryBorder: "${border}",
    primaryText: "${text}",
    primaryOnSoft: "${os}",

    appBg: "${NEUTRAL.appBg}",
    windowBg: "${NEUTRAL.windowBg}",
    panelBg: "${NEUTRAL.panelBg}",
    cardBg: "${NEUTRAL.cardBg}",
    inputBg: "${NEUTRAL.inputBg}",
    mutedBg: "${NEUTRAL.mutedBg}",

    border: "${NEUTRAL.border}",
    borderStrong: "${NEUTRAL.borderStrong}",
    divider: "${NEUTRAL.divider}",
    focusRing: "rgba(${r}, ${g}, ${b}, 0.15)",

    text: "${NEUTRAL.text}",
    bodyText: "${NEUTRAL.bodyText}",
    textSecondary: "${NEUTRAL.textSecondary}",
    muted: "${NEUTRAL.muted}",
    placeholder: "${NEUTRAL.placeholder}",
    disabledText: "${NEUTRAL.disabledText}",

    success: "${NEUTRAL.success}",
    successSoft: "${NEUTRAL.successSoft}",
    warning: "${NEUTRAL.warning}",
    warningSoft: "${NEUTRAL.warningSoft}",
    danger: "${NEUTRAL.danger}",
    dangerSoft: "${NEUTRAL.dangerSoft}",
    info: "${primary}",
    infoSoft: "${soft}",

    shadowWindow:
      "0 12px 32px rgba(27, 39, 52, 0.14), 0 2px 6px rgba(27, 39, 52, 0.07)",
    shadowPanel: "0 1px 2px rgba(27, 39, 52, 0.04)",
    shadowCardHover: "0 1px 2px rgba(27, 39, 52, 0.06)",
    shadowSelected: "none",

    swatch: "${primary}",
  },

`;
}
out = out.replace(/\n\n$/, '\n') + '};';

import('node:fs').then(({default:fs})=>{
  const f='src/theme/theme.ts';
  const s=fs.readFileSync(f,'utf8');
  const start=s.indexOf('export const CLIPLY_THEMES');
  const end=s.indexOf('export const CLIPLY_THEME_OPTIONS');
  fs.writeFileSync(f, s.slice(0,start)+out+'\n\n'+s.slice(end));
  console.table(report);
  console.log('themes written:', defs.length);
});
