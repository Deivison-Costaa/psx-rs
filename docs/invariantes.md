# Invariantes e notas

Referência estável do projeto: comportamento que já custou defeito, e escolha assumida que ainda
não foi verificada contra hardware. **Não é handoff.** O `STATUS.md` cita daqui **por número**, na
linha `Invariantes relevantes:` da próxima tarefa; quem itera lê só as citadas (R8).

A numeração é permanente. Item novo vai para o **fim**; item resolvido é marcado como resolvido no
próprio texto, nunca removido nem renumerado — número que muda invalida toda citação anterior.
Regra imposta por `status_handoff.rs`.

1. BIOS local: `bios/SCPH1001.BIN` (MD5 924E392ED05558FFDB115408C263DCCF), gitignored,
   validada na iter 0009 (item 0.9). Nunca commitar.
2. **Load delay × escrita no mesmo registrador: comportamento ASSUMIDO, não verificado
   (resolve no item 1.11).** Quando a instrução do delay slot escreve o registrador
   destino do load (`lw r10,..` seguido de `ori r10,..`), a nossa implementação faz o
   **load vencer**. A spec local não decide: `02-cpu.md § Caution - Load Delay` só diz que
   o registrador "não é atualizado até o próximo opcode ter completado", o que fala de
   leitura, não de precedência de escrita. Não mudamos sem evidência (R1). O teste
   `load_delay_vs_escrita_no_mesmo_registrador_comportamento_assumido` fixa o que fazemos
   hoje e nomeia a dúvida, para que uma futura mudança seja deliberada. Ponto de
   resolução: Amidog `psxtest_cpu` no item 1.11 — se ele reprovar, inverter a ordem em
   `Cpu::step` (commitar o load antes de executar, escrevendo num banco de saída).
3. **BcondZ com `rt` fora da tabela: comportamento ASSUMIDO (resolve no item 1.11).** O
   opcode 01h só tem `rt`=00h/01h/10h/11h tabelados em `02-cpu.md § Opcode/Parameter
   Encoding`; a spec local não diz o que `rt`=02h..0Fh/12h..1Fh fazem. Assumimos **no-op
   silencioso** (nem desvia nem linka). O teste
   `bcondz_rt_fora_da_tabela_comportamento_assumido` fixa isso e diz na asserção que é
   suposição. Se o Amidog `psxtest_cpu` reprovar, o critério a testar primeiro é o de
   hardware conhecido: bit16 sozinho decide BLTZ/BGEZ e o link ocorre quando os bits
   20..17 valem 1000b — o que faria `rt`=02h agir como BLTZ.
4. **EPC e BadVaddr são graváveis via MTC0 — comportamento ASSUMIDO (resolve no item
   1.11).** A spec marca ambos como (R), mas o comportamento sob escrita não está
   documentado localmente. Implementados como R/W na 1.8a. Os testes
   `epc_gravavel_comportamento_assumido` e `badvaddr_gravavel_comportamento_assumido`
   fixam o comportamento atual. Se o Amidog `psxtest_cpu` reprovar, adicionar `if reg ==
   8 || reg == 14 { return; }` em `cop0_write`.
5. **TAR (cop0r6) é R/W — comportamento ASSUMIDO (resolve no item 1.11).** Mesmo
   critério de EPC/BadVaddr: spec marca (R), implementado como R/W sem evidência
   contrária.
6. **Registradores N/A do COP0 (r0-r2, r4, r10, r32-r63) não disparam exceção — dívida
   do 1.8c, a agendar depois do 1.11.** Leitura retorna 0, escrita é ignorada. O comportamento correto é Reserved
   Instruction Exception (excode=0Ah).
7. **Acesso ao COP0 em User mode com COP0 disabled — dívida do 1.8c, a agendar depois do 1.11.** Acessar qualquer
   registrador do COP0 que não seja garbage (r16-r31), ou executar RFE, em User mode com
   COP0 disabled (SR.bit1=1 e SR.bit28=0) gera Coprocessor Unusable Exception (excode=0Bh).
   Os registradores garbage r16-r31 podem ser acessados nesse estado sem exceção. Fonte:
   `docs/reference/02-cpu.md`, seção cop0r16-r31 - Garbage (L885).
8. **E1 — Entrada de exceção preserva bits Sw (8-9) e IP (10-15) do CAUSE.** A escrita
   do ExcCode agora usa máscara: `self.cop0[13] = (self.cop0[13] & !0xC000_007C) | cause`,
   gravando apenas BD (bit31), BT (bit30) e ExcCode (bits 2-6). O erro original
   (`self.cop0[13] = cause`) zerava os bits Sw, contradizendo a spec que diz "clear them
   before returning from the exception handler" — instrução de software que só faz sentido
   se o hardware não limpar sozinho. Corrigido na revisão adversarial do PR #35.
9. **E2 — Empilhamento de SR na entrada da exceção: comportamento ASSUMIDO (resolve no
   item 1.11).** O inverso exato do RFE: bits 0-1 (IEc/KUc) → bits 2-3 (IEp/KUp), bits 2-3
   (IEp/KUp) → bits 4-5 (IEo/KUo), bits 0-1 zerados. A spec local NÃO documenta o push,
   apenas o RFE que desempilha. Sem o push, um handler que execute RFE restaura lixo nos
   bits IEc/KUc. Ponto de resolução: Amidog `psxtest_cpu`. Testes:
   `sr_e_empilhado_na_entrada_da_excecao` e `sr_push_seguido_de_rfe_restaura_os_bits_0_3`.
10. **E3 — Load delay commitado antes da exceção: comportamento ASSUMIDO (resolve no
   item 1.11).** O acesso à memória do `lw` já ocorreu quando a exceção da instrução
   seguinte é reconhecida; o valor pendente é commitado antes do desvio para o handler.
   A spec local não tem evidência sobre este caso (R1). Escolha (a) entre duas opções
   igualmente plausíveis. Teste:
   `load_pendente_e_commitado_antes_da_excecao_comportamento_assumido`.
11. **A spec se contradiz sobre o EPC do `syscall` — ASSUMIDO (resolve no item 1.11).**
   `cop0r14 - EPC` diz que o registrador guarda "the address at which an exception occured",
   o que dá `EPC = endereço do próprio syscall`; a seção `exception opcodes` descreve o
   handler examinando `[epc-4]` para ler o opcode que causou a exceção, o que só fecha se
   `EPC` apontasse para a instrução **seguinte**. Implementamos a primeira leitura, que é a
   do registrador em si e a que os testes B2/B4 fixam. As duas leituras se reconciliam se
   `[epc-4]` for descuido de redação sobre o caso BD (onde `EPC` de fato aponta 4 bytes
   antes). Não mudamos sem evidência (R1). Ponto de resolução: Amidog `psxtest_cpu`.
12. **`file_size.rs`: `cpu_exception_mechanism.rs` está em 487 linhas de 500.** O próximo
    teste de exceção vai para `cpu_exception_estado_previo.rs`, criado na segunda rodada de
    revisão do PR #35, ou para um arquivo novo com nome próprio. Não corte casos existentes.
    **`spec_citations.rs` está em 500 de 500** — a próxima asserção ali derruba a CI.
13. **Linhas INCLUEM a coordenada inferior-direita (`docs/reference/03-gpu.md` L361-362); polígonos EXCLUEM (`docs/reference/03-gpu.md` L323).**
    As duas funções de rasterização têm regras de borda OPOSTAS. `render_single_line` usa
    Bresenham com break em `x==x1 && y==y1` (inclusivo); `render_triangle` usa
    `xr.min(area_x2 + 1)` (exclusivo). Não reusa uma na outra.
14. **10.22 (mask-bit) verificado completo em 2026-07-30.** Scoreboard no commit bd1a838:
    `mask-bit = pass, 5p/0f`. A iter 0075 fechou os 2 subtestes que falhavam (3p/2f → 5p/0f).
    O STATUS.md da 0080 continha informação incorreta — corrigido na 0081.
15. **Imediato de endereçamento é SINALIZADO.** Todo load/store (`lw/sw/lb/lh/...`) e todo
    `addi/addiu/slti/sltiu` sign-extendem o campo de 16 bits: `(instr & 0xFFFF) as u16 as
    i16 as u32`. Só a família lógica (`andi/ori/xori`) zero-extende. Violado na iter 0011
    (SW), pego na revisão adversarial; qualquer item novo que leia um imediato reconfere
    esta linha antes de escolher a extensão. (Era a única entrada da seção `## Invariantes`
    do STATUS.md; virou a 15 ao ser movida para cá, sem renumerar as outras.)
16. **Vetorar exceção OU interrupção descarta o salto pendente e recua o EPC.** Se a
    instrução preemptada está num delay slot, `EPC` aponta para o **branch** (PC−4),
    `CAUSE.BD` (bit31) é setado e `CAUSE.BT` (bit30) diz se o branch seria tomado
    (`docs/reference/02-cpu.md` L682-683). E `branch_target`/`delay_slot_pending` **têm de ser
    zerados**: salto pendente que sobrevive à vetoração sequestra o PC na primeira instrução do
    handler. O caminho de exceção (`pending_exception`) sempre fez isso; o de **interrupção** era
    um `return` antecipado separado que não fazia nenhuma das duas coisas, e matou o boot da BIOS
    por 8 iterações (item 4.4f, iter 0103). Caminho novo que vetore: passa pelas duas regras.
17. **Load custa mais que 1 ciclo, e o custo depende da regiao.** `docs/reference/02-cpu.md`
    L262-269, medido em hardware: scratchpad 1, I/O on-die 5, RAM principal 7, ROM da BIOS 27..33
    (fixamos 27). Store custa 1 (write-queue, L305-306). Nao e detalhe de precisao: a BIOS espera
    o vblank gastando um orcamento FIXO de 32 768 iteracoes de um laco de 12 instrucoes com 3
    loads da RAM. Com 1 ciclo por instrucao isso cobre 69% de um frame e a espera nunca e
    satisfeita (item 4.4g, iter 0104). Qualquer mudanca no modelo de ciclos reconfere este numero.
18. **GP0(80h) le toda a origem antes de escrever — COMPORTAMENTO ASSUMIDO.** A spec
    (`docs/reference/03-gpu.md` L609-615) nao diz a ordem de varredura do blit, entao regiao de
    origem e destino sobrepostas fica indefinida por ela. Escolhemos ler tudo para um buffer antes
    de escrever, que e o unico resultado independente da ordem; copia in-place da esquerda para a
    direita arrastaria o primeiro pixel. Teste que fixa:
    `origem_e_lida_antes_de_qualquer_escrita_em_regiao_sobreposta`. Ponto de resolucao: suite de
    hardware do ps1-tests para GP0(80h).
19. **Mascarar Xpos na entrada do blit e redundante; mascarar Ypos NAO.** O laco ja faz
    `(x + col) & 0x3FF`, e como `0x400` divide `2^16` a mascara de fora nao muda nada — foi o
    equivalente m3 da bateria 0105. Para Y a conta nao vale: `0x200` nao divide `2^16`, entao
    `& 0x1FF` na entrada e observavel. Quem for otimizar isto nao pode tratar os dois eixos igual.
20. **[RESOLVIDA na 0111 — historico]** A tela de boot esperada, e as hipoteses eliminadas do
    antigo 2.2d. O "Y dobrado" nao era defeito de geometria: era a tela transitoria de um boot
    que morria no 4.4h antes de ligar o modo 480i. Corrigido o load delay (invariante 23), a
    cena aparece inteira. O resto fica como registro do que foi eliminado.
    Texto original: Referencia fornecida pelo usuario
    em 30/07 (tela oficial "SONY COMPUTER ENTERTAINMENT"): fundo **branco**; "SONY" **acima**, em
    azul-escuro; losango **completo**, quatro pontas, dourado/laranja com o "S" vazado, centrado;
    "COMPUTER ENTERTAINMENT" **abaixo**, azul-escuro. Ver `docs/referencias/tela-de-boot.md`.
    Nosso desvio medido: fundo cinza (180,180,180), "SONY" vermelho, losango so pela metade de
    baixo, texto de baixo ausente. Itens 2.2d (geometria), 2.2e (cor), 2.2f (texto ausente).
    O losango ocupa `y=112..368` (centro 240, altura 256) num framebuffer de 640x240, onde o centro
    devia ser 120 e a altura ~128: **Y e exatamente o dobro, X esta certo**.
    **Nao mexer no recorte pela area de desenho** — ele tem teste proprio, esta certo, e e o
    caminho mais tentador para deixar a tela bonita escondendo a causa. A BIOS realmente emite
    256 linhas para uma area de 240 que ela mesma programou; o erro esta ANTES do recorte.
    **Hipoteses ja medidas e ELIMINADAS para o 2.2d — nao repetir:** (a) projecao do GTE — **zero**
    chamadas de `rtps` em 85 M passos, o logo nao passa pelo GTE; (b) resolucao vertical mal
    reportada — `GPUSTAT=0x1406260D`, 640x240 sem entrelacamento, correto; (c) vertice escrito
    direto pela CPU — nenhum `sw` com o valor do vertice, a lista de display vai por **DMA**.
    Proximo passo: interceptar o canal 2 do DMA e ler os pacotes na RAM.
21. **A visualizacao da VRAM como cor engana em regiao de textura 4bpp.** No despejo, a area da
    texpage aparece rosa/azul porque cada halfword vira um pixel de 15 bits; ali cada halfword sao
    **quatro indices de CLUT**, nao uma cor. Ja me levou a suspeitar de cor errada na textura onde
    nao havia. Para julgar cor, olhe o que foi RASTERIZADO, nunca a texpage crua.
22. **[RESOLVIDA na 0111 — historico]** A tela do logo aos 85 M passos era estado INTERMEDIARIO
    de um boot que morria no 4.4h; com o fix da invariante 23 o boot completa, liga 480i e
    desenha a cena inteira. A licao que sobrevive: **tela de boot so se julga com o boot vivo e o
    display ligado.** Medido na 0110: o display fica desligado do inicio ao crash;
    o fundo e um fade `000000 -> B4B4B4` congelado pelo crash (o cinza (180,180,180) da 0109 e o
    ultimo quad do fade, nao bug de cor); e o "SONY" vermelho e CLUT pisoteada: as tres CLUTs do
    texto moram na linha 480 da VRAM (x=192/256/320), DENTRO da area de desenho (0,241)-(639,480)
    da segunda passada, e o losango com Y dobrado (2.2d) rasteriza por cima delas com seu
    gradiente. Modulacao (10.13) esta refutada como causa: os 357 quads texturizados do boot usam
    cor 0x808080, identidade (§ Modulation, 03-gpu.md). A lista da BIOS e uma cena de 480 linhas
    desenhada 2x por frame com offsets (0,1)/(0,241) e display start alternando 1/241 — do nosso
    jeito, as duas metades recebem a MESMA metade superior da cena; o modo final de video e
    incognoscivel ate o boot sobreviver ao 4.4h.
23. **No load delay slot, a escrita da instrucao seguinte VENCE o load pendente.** `lw rX` seguido
    de instrucao que escreve `rX`: o valor do load e descartado (no pipeline, a instrucao emitida
    depois escreve por ultimo). A spec local (§ Caution - Load Delay, 02-cpu.md) so cobre a
    LEITURA no delay slot; a prova da regra de escrita e a propria BIOS SCPH1001: `beq` nao
    tomado com `lw $ra,0x24($sp)` no delay slot seguido de `jal` (0x8004723C-40) — o link do
    `jal` tem de sobreviver, senao o retorno pula um epilogo e o `$sp` fica 0x28 desalinhado por
    65 M passos ate estourar como `$ra=4` (era o item 4.4h). Excecao: IRQ entre o load e a
    instrucao seguinte completa o load antes do handler (§ Caution - Load Delay, L255-257).
    Bateria 0111; teste `cpu_load_delay_escrita_vence.rs`.
