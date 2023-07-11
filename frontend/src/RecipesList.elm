module RecipesList exposing (..)

import Element exposing (..)
import Element.Background
import Element.Border as Border
import Element.Input
import FeatherIcons as FI
import Html exposing (a)
import Http
import Json.Decode as Decode
import Json.Encode as Encode
import Platform.Cmd as Cmd
import RecipeIngredients exposing (..)
import RecipeSteps exposing (Step, StepMsg, fetchSteps, updateRecipeSteps, updateSteps, viewSteps)
import String exposing (pad)
import Test.ExpandableList as ExpandableList exposing (ExpandableList, ExpandableListMsg, mapElementMsg)
import Test.Styles exposing (grey, grey20, white)
import WebData exposing (RemoteData(..), WebData)


type alias RecipesList =
    { recipes : WebData (ExpandableList Recipe RecipeListMsg RecipeMsg)
    , ingredients : WebData (List MetaIngredient)
    , units : WebData (List Unit)
    }


type Recipes
    = Recipes RecipesList


type alias RecipeData =
    { name : String
    , id : Maybe Int
    , comment : Maybe String
    , ingredients : WebData (List RecipeIngredient)
    , steps : WebData (List Step)
    }


type Recipe
    = Recipe { data : RecipeData, edit : RecipeData }


type RecipeListMsg
    = ListMsg (ExpandableListMsg Recipe RecipeMsg)
    | GotWebData WebDataMsg


type WebDataMsg
    = GotRecipes (Result Http.Error (List Recipe))
    | GotRecipeIngredients Int (Result Http.Error (List RecipeIngredient))
    | GotMetaingredients (Result Http.Error (List MetaIngredient))
    | GotUnits (Result Http.Error (List Unit))
    | GotRecipeUpdate RecipeData (Result Http.Error Int)


type RecipeMsg
    = NameChange String
    | CommentChange String
    | RecipeIngredientChange RecipeIngredient RecipeIngredientMsg
    | StepChange (Maybe Step) StepMsg
    | AddIngredient
    | Save
    | Cancel
    | FetchIngredients


stateOf : String -> List ( Bool, Recipe ) -> WebData (ExpandableList Recipe RecipeListMsg RecipeMsg)
stateOf search items =
    let
        filter : String -> Recipe -> Bool
        filter string ingredient =
            case ingredient of
                Recipe { data } ->
                    String.contains (String.toLower string) (String.toLower data.name)
    in
    Success
        { search = search
        , filter = filter
        , items = items
        , viewElement = viewRecipe
        , mapMsg = ListMsg
        , update = updateRecipe
        , add = Just <| always <| newRecipe Nothing "" (Just "")
        , expandItem = Just FetchIngredients
        }


emptyRecipesData : RecipesList
emptyRecipesData =
    { recipes = NotAsked
    , ingredients = NotAsked
    , units = NotAsked
    }


newRecipe : Maybe Int -> String -> Maybe String -> Recipe
newRecipe id name comment =
    let
        data =
            { id = id, name = name, comment = comment, ingredients = Success [], steps = Success [] }
    in
    Recipe { data = data, edit = data }



-- Views


view : RecipesList -> Element RecipeListMsg
view model =
    case model.recipes of
        Success data ->
            ExpandableList.view data

        Failure _ ->
            el [] (text "Failed to load ingredients")

        _ ->
            el [] (text "Loading")


viewRecipe : Attribute RecipeListMsg -> Bool -> Recipe -> Element RecipeListMsg
viewRecipe expand expanded recipe =
    case recipe of
        Recipe { data } ->
            column [ width fill ]
                [ el [ expand, width fill ] (viewRow data)
                , if expanded then
                    viewExpanded recipe

                  else
                    none
                ]


viewRow : RecipeData -> Element msg
viewRow data =
    row [ spaceEvenly, width fill, paddingXY 50 20 ]
        [ el [ width (fillPortion 1) ] (text (Maybe.withDefault "" (Maybe.map String.fromInt data.id)))
        , el [ width (fillPortion 4) ] (text data.name)
        , el [ width (fillPortion 5) ] (text (Maybe.withDefault "" data.comment))
        ]


viewExpanded : Recipe -> Element RecipeListMsg
viewExpanded recipe =
    let
        viewName edit =
            Element.Input.text []
                { onChange = NameChange
                , label = Element.Input.labelAbove [] (text "Name")
                , placeholder = Just (Element.Input.placeholder [] (text "Name"))
                , text = edit.name
                }

        viewComment text edit =
            Element.Input.text []
                { onChange = CommentChange
                , label = Element.Input.labelAbove [] (text "Comment")
                , placeholder = Just (Element.Input.placeholder [] (text "Comment"))
                , text = Maybe.withDefault "" edit.comment
                }

        viewIngredientsBlock edit =
            column
                [ width fill
                , Border.widthEach { top = 1, bottom = 0, left = 0, right = 0 }
                , Border.color grey20
                , paddingXY 0 10
                ]
                [ viewIngredients edit.ingredients
                , Element.Input.button []
                    { onPress = Just AddIngredient
                    , label = el [ paddingXY 30 10 ] (html (FI.toHtml [] FI.plus))
                    }
                ]

        viewStepsBlock edit =
            el
                [ width fill, Border.widthEach { top = 1, bottom = 0, left = 0, right = 0 }, Border.color grey20 ]
                (viewSteps StepChange edit.steps)

        viewButtons text =
            row [ width fill, spacing 25 ]
                [ Element.Input.button [ alignRight ]
                    { onPress = Just Save
                    , label = el [ padding 10 ] <| text "Save"
                    }
                , Element.Input.button [ alignRight ]
                    { onPress = Just Cancel
                    , label = el [ padding 10 ] <| text "Cancel"
                    }
                ]
    in
    case recipe of
        Recipe { edit } ->
            Element.map (ListMsg << ExpandableList.mapElementMsg recipe) <|
                column [ Element.Background.color white, width fill, padding 10, spacing 10, Border.rounded 5 ]
                    [ viewName edit
                    , viewComment text edit
                    , viewIngredientsBlock edit
                    , viewStepsBlock edit
                    , viewButtons text
                    ]


viewIngredients : WebData (List RecipeIngredient) -> Element RecipeMsg
viewIngredients wd =
    case wd of
        Failure _ ->
            text "Failure loading ingredients"

        Success list ->
            column [ width fill ]
                (List.map (\e -> Element.map (RecipeIngredientChange e) (viewRecipeIngredient e)) list)

        _ ->
            text "Loading ingredients"



-- Updates


replaceId : (( Bool, Recipe ) -> ( Bool, Recipe )) -> Int -> List ( Bool, Recipe ) -> List ( Bool, Recipe )
replaceId f id list =
    List.map
        (\( expanded, r ) ->
            case r of
                Recipe { data } ->
                    if data.id == Just id then
                        f ( expanded, r )

                    else
                        ( expanded, r )
        )
        list


updateRecipe : RecipeMsg -> Recipe -> ( Recipe, Cmd RecipeListMsg )
updateRecipe msg rc =
    let
        updateEdit f =
            case rc of
                Recipe i ->
                    let
                        edit =
                            i.edit
                    in
                    Recipe { i | edit = f edit }
    in
    case msg of
        NameChange name ->
            ( updateEdit (\e -> { e | name = name }), Cmd.none )

        CommentChange comment ->
            ( updateEdit (\e -> { e | comment = Just comment }), Cmd.none )

        Save ->
            ( rc, sendRecipe rc )

        Cancel ->
            case rc of
                Recipe { data } ->
                    ( Recipe { data = data, edit = data }, Cmd.none )

        RecipeIngredientChange ri riMsg ->
            let
                updateIfSame =
                    List.map
                        (\r ->
                            if r == ri then
                                updateRecipeIngredient riMsg ri

                            else
                                r
                        )
            in
            case rc of
                Recipe { data, edit } ->
                    ( Recipe
                        { data = data
                        , edit = { edit | ingredients = WebData.map updateIfSame edit.ingredients }
                        }
                    , Cmd.none
                    )

        AddIngredient ->
            case rc of
                Recipe { data, edit } ->
                    let
                        appendNew l =
                            l
                                ++ [ { ingredient = Nothing
                                     , unit = Nothing
                                     , amount = ""
                                     , allIngredients = { list = NotAsked, search = "", hidden = True }
                                     , allUnits = { list = NotAsked, search = "", hidden = True }
                                     }
                                   ]
                    in
                    ( Recipe
                        { data = data
                        , edit = { edit | ingredients = WebData.map appendNew edit.ingredients }
                        }
                    , Cmd.batch
                        [ fetchAllMetaIngredients (GotWebData << GotMetaingredients)
                        , fetchUnits (GotWebData << GotUnits)
                        ]
                    )

        StepChange step stepMsg ->
            case rc of
                Recipe { edit, data } ->
                    let
                        ( steps, cmd ) =
                            updateSteps stepMsg edit.steps step
                    in
                    ( Recipe { data = data, edit = { edit | steps = steps } }, cmd )

        FetchIngredients ->
            case rc of
                Recipe { data } ->
                    ( rc
                    , Maybe.withDefault Cmd.none <|
                        Maybe.map
                            (\id ->
                                Cmd.batch
                                    [ fetchRecipeIngredients (GotWebData << GotRecipeIngredients id) id
                                    , Cmd.map (ListMsg << mapElementMsg rc << StepChange Nothing) <| fetchSteps id
                                    ]
                            )
                            data.id
                    )


handleWebData : WebDataMsg -> RecipesList -> ( RecipesList, Cmd RecipeListMsg )
handleWebData msg model =
    let
        saveUnits : WebData (List Unit) -> RecipeIngredient -> RecipeIngredient
        saveUnits wd ri =
            { ri | allUnits = { list = wd, search = ri.allUnits.search, hidden = True } }

        saveMetaIngredients : WebData (List MetaIngredient) -> RecipeIngredient -> RecipeIngredient
        saveMetaIngredients wd ri =
            { ri | allIngredients = { list = wd, search = ri.allUnits.search, hidden = True } }

        saveInRecipe : (RecipeIngredient -> RecipeIngredient) -> Recipe -> Recipe
        saveInRecipe f r =
            case r of
                Recipe { edit, data } ->
                    case edit.ingredients of
                        Success ig ->
                            Recipe { data = data, edit = { edit | ingredients = Success <| List.map f ig } }

                        _ ->
                            Recipe { data = data, edit = edit }

        noop =
            ( model, Cmd.none )
    in
    case ( msg, model.recipes ) of
        ( GotRecipes result, Success list ) ->
            case result of
                Ok new ->
                    ( { model
                        | recipes = Success { list | items = List.map (Tuple.pair False) new }
                      }
                    , Cmd.none
                    )

                Err e ->
                    ( { model | recipes = Failure e }, Cmd.none )

        ( GotRecipes result, _ ) ->
            case result of
                Ok new ->
                    ( { model | recipes = stateOf "" <| List.map (Tuple.pair False) new }, Cmd.none )

                Err e ->
                    ( { model | recipes = Failure e }, Cmd.none )

        ( GotRecipeIngredients id result, Success list ) ->
            let
                ingredients =
                    case result of
                        Ok i ->
                            Success i

                        Err e ->
                            Failure e

                saveIngredients =
                    \( e, r ) ->
                        case r of
                            Recipe { data, edit } ->
                                ( e, Recipe { data = { data | ingredients = ingredients }, edit = { edit | ingredients = ingredients } } )
            in
            ( { model | recipes = Success { list | items = replaceId saveIngredients id list.items } }
            , Cmd.batch
                [ fetchAllMetaIngredients (GotWebData << GotMetaingredients)
                , fetchUnits (GotWebData << GotUnits)
                ]
            )

        ( GotMetaingredients result, Success list ) ->
            case result of
                Ok meta ->
                    ( { model
                        | recipes = Success { list | items = List.map (Tuple.mapSecond <| saveInRecipe <| saveMetaIngredients <| Success meta) list.items }
                        , ingredients = Success meta
                      }
                    , Cmd.none
                    )

                Err _ ->
                    noop

        ( GotUnits result, Success list ) ->
            case result of
                Ok units ->
                    ( { model
                        | recipes = Success { list | items = List.map (Tuple.mapSecond <| saveInRecipe <| saveUnits <| Success units) list.items }
                        , units = Success units
                      }
                    , Cmd.none
                    )

                Err _ ->
                    noop

        ( GotRecipeUpdate rc result, Success list ) ->
            case result of
                Ok id ->
                    let
                        successfulEdit r =
                            case r of
                                Recipe { edit } ->
                                    if edit == rc then
                                        let
                                            new =
                                                Debug.log "update" { edit | id = Just (Debug.log "id" id) }
                                        in
                                        Recipe { edit = new, data = new }

                                    else
                                        r

                        recipe =
                            list.items
                                |> List.filter
                                    (\( _, r ) ->
                                        case r of
                                            Recipe { edit } ->
                                                edit == rc
                                    )
                                |> List.head
                                |> Maybe.map Tuple.second
                                |> Maybe.map successfulEdit

                        mapStepCmd cmd =
                            recipe
                                |> Maybe.map (\r -> Cmd.map (ListMsg << mapElementMsg r << StepChange Nothing) cmd)
                                |> Maybe.withDefault Cmd.none
                    in
                    ( { model
                        | recipes = Success { list | items = List.map (Tuple.mapSecond successfulEdit) list.items }
                      }
                    , Cmd.batch
                        [ sendIngredients rc
                        , mapStepCmd (updateRecipeSteps rc.steps id)
                        ]
                    )

                _ ->
                    noop

        ( GotRecipeUpdate _ _, _ ) ->
            noop

        ( GotRecipeIngredients _ _, _ ) ->
            noop

        ( GotMetaingredients _, _ ) ->
            noop

        ( GotUnits _, _ ) ->
            noop


update : RecipeListMsg -> RecipesList -> ( RecipesList, Cmd RecipeListMsg )
update msg model =
    case ( msg, model.recipes ) of
        ( ListMsg m, Success data ) ->
            Tuple.mapFirst
                (\result -> { model | recipes = Success result })
                (ExpandableList.update m data)

        ( GotWebData wd, _ ) ->
            handleWebData wd model

        _ ->
            ( model, Cmd.none )



-- Decoding


decodeRecipe : Decode.Decoder Recipe
decodeRecipe =
    Decode.map3
        (\id name comment ->
            let
                data =
                    { id = id, name = name, comment = comment, ingredients = NotAsked, steps = NotAsked }
            in
            Recipe { data = data, edit = data }
        )
        (Decode.field "recipe_id" (Decode.nullable Decode.int))
        (Decode.field "name" Decode.string)
        (Decode.field "comment" (Decode.nullable Decode.string))


decodeRecipes : Decode.Decoder (List Recipe)
decodeRecipes =
    Decode.list decodeRecipe



-- Fetching


fetchRecipes : Cmd RecipeListMsg
fetchRecipes =
    Http.get
        { url = "http://localhost:3000/recipes/list"
        , expect = Http.expectJson (GotWebData << GotRecipes) decodeRecipes
        }



-- Encoding


encodeRecipeData : RecipeData -> Encode.Value
encodeRecipeData rd =
    Encode.object
        [ ( "recipe_id", rd.id |> Maybe.map Encode.int |> Maybe.withDefault Encode.null )
        , ( "name", Encode.string rd.name )
        , ( "comment", rd.comment |> Maybe.map Encode.string |> Maybe.withDefault Encode.null )
        ]


encodeRecipe : Recipe -> Encode.Value
encodeRecipe r =
    case r of
        Recipe { edit } ->
            encodeRecipeData edit



-- Sending


sendRecipe : Recipe -> Cmd RecipeListMsg
sendRecipe r =
    let
        url =
            case r of
                Recipe { data } ->
                    case data.id of
                        Just id ->
                            "/" ++ String.fromInt id ++ "/update"

                        Nothing ->
                            "/create"
    in
    case r of
        Recipe { edit } ->
            Http.post
                { url = "http://localhost:3000/recipes" ++ url
                , body = Http.jsonBody <| encodeRecipe r
                , expect = Http.expectJson (GotWebData << GotRecipeUpdate edit) Decode.int
                }


sendIngredients : RecipeData -> Cmd RecipeListMsg
sendIngredients rd =
    case Debug.log "enc" ( rd.id, encodeIngredients rd.ingredients ) of
        ( Just id, Just body ) ->
            Http.post
                { url = "http://localhost:3000/recipes/" ++ String.fromInt id ++ "/meta_ingredients/update"
                , body = Http.jsonBody body
                , expect = Http.expectJson (GotWebData << GotRecipeUpdate rd) (Decode.succeed 0)
                }

        _ ->
            Cmd.none
