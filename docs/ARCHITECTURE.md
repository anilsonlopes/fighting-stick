# Arquitetura

- `fighting-stick-core`: perfil serializável, validação de duplicatas, Learn e máquina de estados Note On/Off.
- `fighting-stick-input`: detecção pela API de joystick bruto do SDL2, evitando depender de um perfil GameControllerDB completo. Botões, hats e eixos são normalizados; eixos adicionais também podem ser aprendidos.
- `fighting-stick-midi`: enumeração, conexão e envio à porta MIDI escolhida.
- `fighting-stick-desktop`: interface egui, persistência e coordenação dos componentes.

O núcleo não depende de SDL, MIDI ou interface. Isso permite testar transições e persistência sem hardware. Desconexão, troca de porta, restauração e encerramento passam pela rotina de silêncio: Note Off individual para cada nota ativa seguido do CC 123 (All Notes Off) no canal 1.
