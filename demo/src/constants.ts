import type { CustomOptions } from "./types";

export const INITIAL_CONTENT = `# [선비][1]: 韓國語를 爲한 SmartyPants

[선비][1]는 韓國 國立國語院의 <<[한글 맞춤法][2]>> 또는 北朝鮮의
<<朝鮮말規範集>>([PDF][3])에서 定한 句讀法에 맞도록 글의 句讀點 等의 使用을
校正해주는 HTML 前處理器이다.
([SmartyPants]가 英語에 對해 해주는 處理와 비슷하다.)

또한, \`ko-Kore\` 텍스트, 卽, 國漢文混用體를 \`ko-Hang\` 텍스트, 卽, 한글專用으로도
變換한다.

선비는 Haskell 라이브러리 또는 CLI 또는 HTTP API로 쓸 수 있으며, 어떤 方式이든
아래의 變換을 한다 (各各은 켜고 끌 수 있다).

- 모든 漢字語(例: \`漢字\`)를 한글로 (例: \`한자\`).
- 直線形 따옴標(\`"\` 및  \`'\`)를 曲線形 따옴標(\`"\`·\`"\` 및 \`'\`·\`'\`)로.
- 連달아 찍은 마침標(\`...\`)를 말줄임標(\`…\`)로.
- 고리點(\`。\`) 및 모點(\`、\`)을 온點(\`.\`) 및 半點(\`,\`)으로.
- 數學 不等號 짝(\`<\`와 \`>\`)을 제대로 된 홑화살括弧 짝(\`〈\`와 \`〉\`)으로.
- 두 겹의 數學 不等號 짝(\`<<\`와 \`>>\`)을 제대로 된 겹화살括弧 짝(\`《\`와
  \`》\`)으로.
- 空白으로 둘러싸인 하이픈(\`-\`)이나 한글 母音 으(\`ㅡ\`), 또는 둘이나 세 番
  連續된 하이픈(\`--\`이나 \`---\`)을 제대로 된 줄標(\`—\`)로.
- "보다 작다" 不等號와 이어지는 하이픈 또는 等號(\`<-\`, \`<=\`)를 왼쪽 화살標(\`←\`,
  \`⇐\`)로.
- 하이픈 또는 等號와 이어지는 "보다 크다" 不等號(\`->\` \`=>\`)를 오른쪽 화살標(\`→\`,
  \`⇒\`)로.
- 不等號로 둘러싸인 하이픈 또는 等號(\`<->\`, \`<=>\`)를 양쪽 화살標(\`↔\`, \`⇔\`)로.

變換은 HTML 水準에서 이뤄지므로, CommonMark나 Markdown, Textile 等의 마크업
言語와도 잘 붙는다.  SmartyPants와 마찬가지로, 文字 그대로 解釋되어야 하는
\`<pre>\`·\`<code>\`·\`<script>\`·\`<kbd>\` 같은 HTML 태그 안쪽은 變換되지 않는다.

[1]: https://github.com/dahlia/seonbi
[2]: http://kornorms.korean.go.kr/regltn/regltnView.do
[3]: https://upload.wikimedia.org/wikipedia/commons/0/0b/%EC%A1%B0%EC%84%A0%EB%A7%90%EA%B7%9C%EB%B2%94%EC%A7%91%282010%29.pdf
[SmartyPants]: https://daringfireball.net/projects/smartypants/
`;

export const DEFAULT_CUSTOM_OPTIONS: CustomOptions = {
  quote: "CurvedQuotes",
  cite: "AngleQuotes",
  arrow: { bidirArrow: true, doubleArrow: true },
  ellipsis: true,
  emDash: true,
  stop: "Horizontal",
  hanja: {
    rendering: "DisambiguatingHanjaInParentheses",
    reading: {
      initialSoundLaw: true,
      useDictionaries: new Set(["kr-stdict"]),
    },
  },
};
