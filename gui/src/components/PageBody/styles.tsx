import styled from 'styled-components'
import { PageText } from '../helpers/PageText'
import { PageItemWrapper } from '../helpers/ItemWrapper'

export const Container = styled.div`
    display: flex;
    padding: 1em;
`

export const LeftContainer = styled.aside`
    height: 80vh;
    width: 18vw;
    border-right: 2px solid #ddd;

    @media (max-width: 650px) {
        display: none;
    }
`

export const RightContainer = styled.section`
    height: 80vh;
    width: 82vw;
    display: flex;
    flex-direction: column;
    margin-left: 1.5em;
`
export const Image = styled.img`
    width: 13em;
`
export const Text = styled(PageText)`
    color: ${(props) => (props.color ? props.color : '#131A22')};
    font-size: ${(props) => (props.fontSize ? props.fontSize : '.9em')};
`
export const BoldText = styled(Text)`
    font-weight: bold;
    padding: 0.4em;
`

export const Paragraph = styled.p`
    font-size: 0.9em;
    display: flex;
    align-items: center;
    padding-bottom: 0.1em;
`

export const Wrapper = styled(PageItemWrapper)`
    display: flex;
    margin-right: 1em;
    flex-direction: ${(props) => (props.flexDirection ? props.flexDirection : 'row')};
    align-items: ${(props) => (props.alignItems ? props.alignItems : 'left')};
    margin: ${(props) => (props.margin ? props.margin : '')};
`

export const IconWrapper = styled.div`
    color: #ff9900;
`
