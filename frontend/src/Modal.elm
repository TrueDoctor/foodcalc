module Modal exposing (..)

import FeatherIcons as FI
import Html
import Html.Attributes
import Html.Events


viewModal : String -> msg -> List (Html.Html msg) -> List (Html.Html msg) -> Html.Html msg
viewModal title onClose footer content =
    Html.node "dialog"
        [ Html.Attributes.attribute "open" "" ]
        [ Html.header []
            [ Html.a [ Html.Events.onClick onClose, Html.Attributes.href "#" ] [ FI.toHtml [] FI.x ]
            , Html.text title
            ]
        , Html.p [ Html.Attributes.class "container" ] content
        , Html.footer [] footer
        ]
