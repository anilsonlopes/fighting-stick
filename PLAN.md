# Fighting Stick Mini -> Ableton Live Lite MIDI Controller

## Resumo

Criar um app Windows independente — não um dispositivo Max for Live — que converte o Fighting Stick Mini em MIDI para o Ableton Live 12 Lite através de uma porta virtual (loopMIDI). Isso contorna a limitação central: Max for Live é incluído no Live Suite e é add-on no Standard, não no Lite. [Ableton](https://help.ableton.com/hc/en-us/articles/360000036850-Max-for-Live-bundled-in-Live)

A inspiração será o fluxo dos dispositivos Gamepad Control e GAMEPAD M4L: mapeamento configurável, modo Learn e performance direta. A primeira versão restringe-se ao que existe no arcade stick: botões e direções, sem gyro, trackpad ou LEDs. [Side Brain](https://sidebrain.net/gamepad-control/), [User Friendly](https://maxforlive.com/library/device/13165/gamepad-maxforlive-by-user-friendly)

## Arquitetura

- Projeto Rust em workspace, entregue inicialmente como código-fonte:
  - `crates/core`: perfis, atribuições, validação e máquina de estados MIDI.
  - `crates/input`: SDL2 Game Controller API para detectar o stick e normalizar botões/direções.
  - `crates/midi`: saída MIDI para a porta loopMIDI selecionada.
  - `apps/desktop`: interface Windows em egui/eframe.
  - `docs`: instalação, uso no Live, pesquisa e troubleshooting.
- Usar o identificador SDL (GUID + nome) para selecionar e persistir um perfil por controle. SDL permite complementar mapeamentos por `gamecontrollerdb.txt` quando um modelo Hori não for reconhecido corretamente. [Cycling ’74: `gamepad`](https://docs.cycling74.com/reference/gamepad/)
- Salvar os perfis em `%APPDATA%\\FightingStickMidi`, incluindo porta MIDI escolhida, dispositivo selecionado, mapeamentos e velocidade padrão.
- Tela única com: estado do dispositivo, seleção da porta MIDI, grade de mapeamento, Learn por controle, teste visual de entradas e salvar/restaurar perfil.

## Comportamento MIDI

- Configuração inicial: 12 pads em notas consecutivas da grade padrão de Drum Rack (C1–B1 / MIDI 36–47):
  - quatro direções da alavanca;
  - oito botões de ação.
- Cada entrada envia `Note On` ao pressionar e `Note Off` ao soltar, com velocidade padrão 100.
- O Learn substitui a entrada física e/ou a nota de destino. Notas duplicadas no mesmo perfil serão bloqueadas para evitar `Note Off` prematuro.
- Direções simultâneas permanecem independentes, permitindo disparar dois pads.
- Ao perder conexão, trocar de perfil ou fechar o app: emitir `Note Off` para toda nota ativa e `All Notes Off`.
- Start, Select, Home e controles adicionais ficam visíveis no diagnóstico, mas não são mapeados por padrão na v1.

## Integração com Live Lite

- Instalar e criar uma porta no loopMIDI, por exemplo `Fighting Stick MIDI`.
- No Live 12 Lite: habilitar essa porta como entrada MIDI, criar faixa MIDI, inserir Drum Rack e armar/monitorar a faixa.
- O app não automatiza transporte, clips ou parâmetros do Live nesta versão; isso evita depender de Max for Live e mantém o uso focado em performance de bateria.
- Futuro opcional: um `.amxd` de painel/controle poderá receber a mesma porta MIDI quando houver Max for Live. O objeto `gamepad` do Max suporta eventos de eixos, botões e direções via SDL2. [Cycling ’74](https://docs.cycling74.com/reference/gamepad/)

## Testes e critérios de aceite

- Testes unitários para Learn, persistência de perfis, validação de notas, transições pressionar/soltar e limpeza em desconexão.
- Testes com fonte de eventos simulados para confirmar a sequência MIDI correta.
- Validação manual no Windows 11 com o Hori conectado: identificação do dispositivo, configuração completa via Learn, disparo dos 12 pads no Drum Rack e ausência de notas presas após desconectar.
- Documentar como capturar nome/GUID e adicionar um mapeamento SDL caso o modelo Hori desconhecido não exponha os controles esperados.

## Premissas

- Windows 11 é o ambiente de uso.
- loopMIDI é uma dependência externa aceita.
- O modelo exato do Fighting Stick Mini será identificado no primeiro teste; a v1 não dependerá de uma tabela fixa de botões.
- A entrega inicial é código-fonte, não instalador nem `.exe`.
