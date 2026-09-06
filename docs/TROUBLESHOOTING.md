# Solução de problemas e mapeamento SDL

## O controle não aparece

Confirme no Windows (`joy.cpl`) que ele é reconhecido. Troque o modo físico do controle entre PC/XInput, PS3 ou PS4, quando houver, e reconecte-o. O app abre o primeiro joystick exposto pelo SDL2 e lê os controles brutos, sem exigir um mapeamento GameControllerDB completo.

## Descobrir nome, GUID e eventos

Execute o app pelo terminal e abra **Diagnóstico de entradas**. Ao conectar, o nome é mostrado no topo; eventos de todos os botões, inclusive Start, Select e Home, aparecem no diagnóstico. O arquivo do perfil em `%APPDATA%\FightingStickMidi` usa o identificador SDL no nome e contém o nome completo.

## Adicionar um mapeamento desconhecido

Use a ferramenta `controllermap.exe` da distribuição de desenvolvimento do SDL para gerar uma linha no formato da SDL GameControllerDB. Salve-a em um arquivo `gamecontrollerdb.txt`. Uma linha contém GUID, nome, plataforma e a associação de eixos/botões. Para contribuir um mapeamento validado, consulte o projeto [SDL_GameControllerDB](https://github.com/mdqinc/SDL_GameControllerDB).

O app usa eventos brutos de joystick, portanto normalmente não precisa de uma entrada GameControllerDB. O mapeamento externo continua útil para testar o controle em outros programas baseados na API Game Controller do SDL.

## Sem som no Live

Confira se a porta ainda está ativa no loopMIDI, se **Track** está habilitado para a entrada e se a faixa está armada ou em Monitor In. Escolha novamente a porta depois de recriá-la.
