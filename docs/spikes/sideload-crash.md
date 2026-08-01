# Spike — sideload do executável do Crash Bandicoot (2026-08-01)

Pergunta: pulando shell + driver de CD, qual é o próximo muro no caminho do menu — GTE?

## Método

1. Extração ISO9660 direto do `.bin` (script `extrai-exe.ps1` no scratchpad):
   `SYSTEM.CNF` (66 B) confirma `BOOT = cdrom:\SCUS_949.00;1`, `STACK = 801FFFF0`;
   `SCUS_949.00` (290816 B) com header PS-X EXE válido: entry `0x8003E018`,
   load `0x80010000`, size `0x46800`, SP `0x801FFFF0`.
2. Andaime local NÃO COMMITADO no `psx-cli` (`--exe-at N`): boot real BIOS+disco por
   385M passos, espera do laço idle do kernel (PC=0xA0) para não injetar dentro de ISR,
   `load_psexe` e +215M passos. Sem `install_return_stubs` (kernel real presente).
   Andaime removido após a medição (precedente: stub de medição da 2.1/A6).

## Resultado

- **Sideload v1 (o `--exe` atual, sem kernel) é inválido para jogo real**: TTY 0 bytes,
  PCs varrendo RAM baixa zerada — o EXE chama o kernel e cai em memória vazia. O braço
  `--exe` nem chama `insert_disc()`. Serve só para EXEs sintéticos de teste.
- **Sideload v2 (injeção pós-kernel): o jogo RODA.** TTY após a injeção:
  `CD_init:addr=800558b0` → `PS-X Control PAD Driver Ver 3.0` (×2 pads) →
  `ResetGraph:jtb=80054a24,env=80054a6c` → **`VSync: timeout`**.
- Depois do `VSync: timeout`: **100% das amostras de PC em `0x80000080`** (vetor de
  exceção) da amostra 33M até 215M — tempestade/laço de exceção. VRAM 100% zerada
  (o jogo limpou e nunca desenhou).

## Resposta à pergunta do spike

**O GTE ainda NÃO é o próximo muro.** Entre o CD-ROM e o GTE existe um muro antes:
o caminho de VSync/IRQ0 **do jogo** (LIBGPU espera o evento de VSync e estoura timeout;
em seguida o vetor de exceção vira residência). O caminho do shell funciona (0080/0129);
o do jogo não — candidatos: entrega/ack de IRQ0 na cadeia de eventos que a LIBETC/LIBGPU
instala, e o defeito já auditado da fase do VBLANK calculada uma única vez em `Bus::new`
(nunca recalculada quando `ResetGraph`/GP1 reprograma o display).

## Consequência para o plano

- Fase B (motor de respostas do CD-ROM) continua necessária para o boot REAL (o loader
  caiu no fallback `PSX.EXE` porque SYSTEM.CNF veio ilegível — seek não avança).
- Novo item-pai candidato DEPOIS do motor: **VSync/IRQ do jogo** (nomeado, com o spike
  como evidência) — antes de qualquer aposta em GTE.
- O sideload pós-kernel é viável como harness de validação parallela barato (repetir
  após cada avanço: o ponto de trava do jogo é um marcador de progresso melhor que o
  placar de testes).
