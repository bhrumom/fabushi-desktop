# fcitx5-qt Qt5 DBus/control deterministic read — 7,485–7,505

Exact component: `fcitx/fcitx5-qt@0285a5d18367d8f3af80dab7e4819d5555982340`.

The Qt5 DBus/control plane is fully read for this range. Build/export remains a platform/release concern. Input session behavior preserves service-owner fencing, async context creation/failure, cleanup, optional action introspection and virtual-keyboard query/event reconciliation. Controller behavior exposes input-source/layout/group/config state plus refresh/reload/restart/update capabilities. Fabushi maps these to existing canonical Settings + platform/input lifecycle owners; Fcitx addon/config records must not become a second Plugins/Marketplace truth. All rows remain mapped-open and close no unknown.

Next: 7,506 `Telegram/ThirdParty/fcitx5-qt::qt5/guiwrapper/CMakeLists.txt@a459b1b1a362d8d485b7a794f27d07ead7a3bf27`.
